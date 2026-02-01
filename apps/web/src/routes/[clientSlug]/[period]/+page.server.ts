import {
	getClientBySlug,
	getProjectsByClientId,
	getSessionsInMonth,
	getCommitsBySessionIds,
	getWorkItemsByProjectIds,
	updateWorkItem,
	db
} from '$lib/server/db';
import { calculateBillableHours, calculateBillableSeconds } from '$lib/billable';
import { error, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';
import type { Session, Commit, Project, WorkItem } from '$lib/server/db';

interface SessionWithDetails extends Session {
	project: Project;
	commits: Commit[];
	/** True if start_commit != end_commit but no commits recorded (likely a bug) */
	maybeMissingCommits: boolean;
}

export const load: PageServerLoad = async ({ params }) => {
	const client = await getClientBySlug(params.clientSlug);
	if (!client) {
		throw error(404, 'Client not found');
	}

	// Parse period (YYYY-MM)
	const periodMatch = params.period.match(/^(\d{4})-(\d{2})$/);
	if (!periodMatch) {
		throw error(400, 'Invalid period format. Expected YYYY-MM');
	}

	const year = parseInt(periodMatch[1], 10);
	const month = parseInt(periodMatch[2], 10);

	if (month < 1 || month > 12) {
		throw error(400, 'Invalid month');
	}

	const projects = await getProjectsByClientId(client.id);
	const projectIds = projects.map((p) => p.id);
	const projectMap = new Map(projects.map((p) => [p.id, p]));

	// Get work items for all projects
	const workItems = await getWorkItemsByProjectIds(projectIds);
	// Create a map keyed by "projectId:identifier" for quick lookup
	const workItemMap = new Map<string, WorkItem>();
	for (const wi of workItems) {
		workItemMap.set(`${wi.project_id}:${wi.identifier}`, wi);
	}

	const sessions = await getSessionsInMonth(projectIds, year, month);
	const sessionIds = sessions.map((s) => s.id);
	const commits = await getCommitsBySessionIds(sessionIds);

	// Group commits by session
	const commitsBySession = new Map<number, Commit[]>();
	for (const commit of commits) {
		const list = commitsBySession.get(commit.session_id) ?? [];
		list.push(commit);
		commitsBySession.set(commit.session_id, list);
	}

	// Enrich sessions with project and commits
	const sessionsWithDetails: SessionWithDetails[] = sessions.map((session) => {
		const sessionCommits = commitsBySession.get(session.id) ?? [];
		// Flag sessions where commits might be missing:
		// start_commit and end_commit exist and differ, but no commits recorded
		const maybeMissingCommits =
			session.start_commit !== null &&
			session.end_commit !== null &&
			session.start_commit !== session.end_commit &&
			sessionCommits.length === 0;

		return {
			...session,
			project: projectMap.get(session.project_id)!,
			commits: sessionCommits,
			maybeMissingCommits
		};
	});

	// Calculate totals
	const totalSeconds = sessions.reduce((sum, s) => sum + (s.active_seconds ?? 0), 0);
	const totalHours = totalSeconds / 3600;
	const totalBillableHours = calculateBillableHours(totalSeconds);
	const totalBillableSeconds = calculateBillableSeconds(totalSeconds);
	const totalSessions = sessions.length;
	const totalCommits = commits.length;

	// Group by branch (work_item is derived from branch, so branch is the primary key)
	const workItemStats = new Map<
		string,
		{
			branch: string;
			workItem: string; // Display name: work_item if available, otherwise branch
			workItemId: number | null; // Database ID for updates
			title: string | null;
			description: string | null;
			completedDate: string | null;
			sessions: SessionWithDetails[];
			totalSeconds: number;
			billableHours: number;
			commits: Commit[];
			lastDate: string | null; // Last session date (YYYY-MM-DD)
		}
	>();

	for (const session of sessionsWithDetails) {
		// Group by branch, display as work_item if available
		const groupKey = session.branch;
		const displayName = session.work_item ?? session.branch;
		// Look up the work item from database (using work_item identifier if available)
		const workItemIdentifier = session.work_item ?? session.branch;
		const dbWorkItem = workItemMap.get(`${session.project_id}:${workItemIdentifier}`);

		const existing = workItemStats.get(groupKey) ?? {
			branch: groupKey,
			workItem: displayName,
			workItemId: dbWorkItem?.id ?? null,
			title: dbWorkItem?.title ?? null,
			description: dbWorkItem?.description ?? null,
			completedDate: dbWorkItem?.completed_date ?? null,
			sessions: [],
			totalSeconds: 0,
			billableHours: 0,
			commits: [],
			lastDate: null
		};
		existing.sessions.push(session);
		existing.totalSeconds += session.active_seconds ?? 0;
		existing.commits.push(...session.commits);
		// Track the latest date (use ended_at if available, otherwise started_at)
		const sessionDate = (session.ended_at ?? session.started_at).slice(0, 10);
		if (!existing.lastDate || sessionDate > existing.lastDate) {
			existing.lastDate = sessionDate;
		}
		workItemStats.set(groupKey, existing);
	}

	// Group by date
	const dailyStats = new Map<
		string,
		{
			date: string;
			sessions: SessionWithDetails[];
			totalSeconds: number;
		}
	>();

	for (const session of sessionsWithDetails) {
		const date = session.started_at.slice(0, 10); // YYYY-MM-DD
		const existing = dailyStats.get(date) ?? {
			date,
			sessions: [],
			totalSeconds: 0
		};
		existing.sessions.push(session);
		existing.totalSeconds += session.active_seconds ?? 0;
		dailyStats.set(date, existing);
	}

	// Minimum display threshold: 5 minutes (300 seconds)
	const MIN_DISPLAY_SECONDS = 300;

	// Sort by date descending, filter out days/sessions below threshold
	const dailyStatsList = Array.from(dailyStats.values())
		.map((day) => ({
			...day,
			sessions: day.sessions.filter((s) => (s.active_seconds ?? 0) >= MIN_DISPLAY_SECONDS)
		}))
		.filter((day) => day.totalSeconds >= MIN_DISPLAY_SECONDS)
		.sort((a, b) => b.date.localeCompare(a.date));

	// Filter out work items below threshold and calculate billable hours
	const allWorkItems = Array.from(workItemStats.values())
		.filter((item) => item.totalSeconds >= MIN_DISPLAY_SECONDS)
		.map((item) => ({
			...item,
			billableHours: calculateBillableHours(item.totalSeconds),
			sessionCount: item.sessions.length
		}));

	// Split into completed (has title AND completed_date) vs tracking
	const completedWorkItems = allWorkItems
		.filter((item) => item.title && item.completedDate)
		.sort((a, b) => {
			// Sort by completed date descending
			if (a.completedDate && b.completedDate) {
				return b.completedDate.localeCompare(a.completedDate);
			}
			return b.totalSeconds - a.totalSeconds;
		});

	const trackingWorkItems = allWorkItems
		.filter((item) => !item.title || !item.completedDate)
		.sort((a, b) => b.totalSeconds - a.totalSeconds);

	// Calculate billable total only from completed items
	const completedBillableHours = completedWorkItems.reduce((sum, item) => sum + item.billableHours, 0);

	// Calculate prev/next month periods
	const prevMonth = month === 1 ? 12 : month - 1;
	const prevYear = month === 1 ? year - 1 : year;
	const nextMonth = month === 12 ? 1 : month + 1;
	const nextYear = month === 12 ? year + 1 : year;

	return {
		client,
		year,
		month,
		projects,
		sessionsWithDetails,
		totalHours,
		totalBillableHours,
		completedBillableHours,
		totalSessions,
		totalCommits,
		completedWorkItems,
		trackingWorkItems,
		// Keep backward compat
		workItemStats: allWorkItems,
		dailyStats: dailyStatsList,
		// Month navigation
		prevPeriod: `${prevYear}-${String(prevMonth).padStart(2, '0')}`,
		nextPeriod: `${nextYear}-${String(nextMonth).padStart(2, '0')}`
	};
};

export const actions: Actions = {
	updateWorkItem: async ({ request }) => {
		const formData = await request.formData();
		const id = Number(formData.get('id'));
		const title = formData.get('title') as string | null;
		const description = formData.get('description') as string | null;

		if (!id || isNaN(id)) {
			return fail(400, { error: '無效的工作項 ID' });
		}

		try {
			const updated = await updateWorkItem(id, {
				title: title || null,
				description: description || null
			});

			if (!updated) {
				return fail(404, { error: '找不到工作項' });
			}

			return { success: true, workItem: updated };
		} catch (e) {
			console.error('Failed to update work item:', e);
			return fail(500, { error: '更新失敗' });
		}
	}
};
