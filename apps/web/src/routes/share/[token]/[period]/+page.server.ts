import {
	getClientByShareToken,
	getProjectsByClientId,
	getSessionsInMonth,
	getCommitsBySessionIds,
	getWorkItemsByProjectIds,
	getCompletedWorkItemsInMonth
} from '$lib/server/db';
import { calculateBillableHours } from '$lib/billable';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { Session, Commit, Project, WorkItem } from '$lib/server/db';

interface SessionWithDetails extends Session {
	project: Project;
	commits: Commit[];
}

export const load: PageServerLoad = async ({ params }) => {
	// Look up client by share token
	const client = await getClientByShareToken(params.token);
	if (!client) {
		throw error(404, '分享連結不存在');
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
	const sessionsWithDetails: SessionWithDetails[] = sessions.map((session) => ({
		...session,
		project: projectMap.get(session.project_id)!,
		commits: commitsBySession.get(session.id) ?? []
	}));

	// Calculate totals
	const totalSeconds = sessions.reduce((sum, s) => sum + (s.active_seconds ?? 0), 0);
	const totalHours = totalSeconds / 3600;

	// Group by branch
	const workItemStats = new Map<
		string,
		{
			branch: string;
			workItem: string;
			workItemId: number | null;
			title: string | null;
			description: string | null;
			completedDate: string | null;
			billableHoursOverride: number | null;
			sessions: SessionWithDetails[];
			totalSeconds: number;
			billableHours: number;
		}
	>();

	for (const session of sessionsWithDetails) {
		const groupKey = session.branch;
		const displayName = session.work_item ?? session.branch;
		const workItemIdentifier = session.work_item ?? session.branch;
		const dbWorkItem = workItemMap.get(`${session.project_id}:${workItemIdentifier}`);

		const existing = workItemStats.get(groupKey) ?? {
			branch: groupKey,
			workItem: displayName,
			workItemId: dbWorkItem?.id ?? null,
			title: dbWorkItem?.title ?? null,
			description: dbWorkItem?.description ?? null,
			completedDate: dbWorkItem?.completed_date ?? null,
			billableHoursOverride: dbWorkItem?.billable_hours ?? null,
			sessions: [],
			totalSeconds: 0,
			billableHours: 0
		};
		existing.sessions.push(session);
		existing.totalSeconds += session.active_seconds ?? 0;
		workItemStats.set(groupKey, existing);
	}

	// Minimum display threshold: 5 minutes (300 seconds)
	const MIN_DISPLAY_SECONDS = 300;

	// Filter and calculate billable hours - session-based completed items
	const sessionBasedCompleted = Array.from(workItemStats.values())
		.filter((item) => item.totalSeconds >= MIN_DISPLAY_SECONDS)
		.filter((item) => item.title && item.completedDate)
		.map((item) => ({
			...item,
			// Use override if set, otherwise calculate
			billableHours: item.billableHoursOverride ?? calculateBillableHours(item.totalSeconds)
		}));

	// Get standalone work items (completed in this month but not tied to sessions)
	const standaloneWorkItems = await getCompletedWorkItemsInMonth(projectIds, year, month);
	const sessionWorkItemIds = new Set(
		Array.from(workItemStats.values()).map((item) => item.workItemId).filter(Boolean)
	);
	const standaloneCompleted = standaloneWorkItems
		.filter((wi) => !sessionWorkItemIds.has(wi.id))
		.map((wi) => ({
			branch: wi.identifier,
			workItem: wi.identifier,
			workItemId: wi.id,
			title: wi.title,
			description: wi.description,
			completedDate: wi.completed_date,
			billableHoursOverride: wi.billable_hours,
			sessions: [] as SessionWithDetails[],
			totalSeconds: 0,
			billableHours: wi.billable_hours ?? 0
		}));

	// Merge and sort (ascending by date - oldest first)
	const completedWorkItems = [...sessionBasedCompleted, ...standaloneCompleted].sort((a, b) => {
		if (a.completedDate && b.completedDate) {
			return a.completedDate.localeCompare(b.completedDate); // Ascending (oldest first)
		}
		return (a.billableHours ?? 0) - (b.billableHours ?? 0);
	});

	const completedBillableHours = completedWorkItems.reduce((sum, item) => sum + item.billableHours, 0);

	// Calculate prev/next month periods
	const prevMonth = month === 1 ? 12 : month - 1;
	const prevYear = month === 1 ? year - 1 : year;
	const nextMonth = month === 12 ? 1 : month + 1;
	const nextYear = month === 12 ? year + 1 : year;

	return {
		token: params.token,
		clientName: client.name,
		year,
		month,
		totalHours,
		completedBillableHours,
		completedWorkItems,
		prevPeriod: `${prevYear}-${String(prevMonth).padStart(2, '0')}`,
		nextPeriod: `${nextYear}-${String(nextMonth).padStart(2, '0')}`
	};
};
