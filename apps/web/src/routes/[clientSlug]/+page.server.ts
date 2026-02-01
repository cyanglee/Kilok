import {
	getClientBySlug,
	getProjectsByClientId,
	getContractByClientAndYear,
	db
} from '$lib/server/db';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params }) => {
	const client = await getClientBySlug(params.clientSlug);
	if (!client) {
		throw error(404, 'Client not found');
	}

	const projects = await getProjectsByClientId(client.id);
	const currentYear = new Date().getFullYear();
	const contract = await getContractByClientAndYear(client.id, currentYear);

	const projectIds = projects.map((p) => p.id);

	// Get monthly stats for the current year
	const monthlyStats: Array<{
		month: number;
		hours: number;
		sessions: number;
	}> = [];

	for (let month = 1; month <= 12; month++) {
		if (projectIds.length === 0) {
			monthlyStats.push({ month, hours: 0, sessions: 0 });
			continue;
		}

		const monthStart = `${currentYear}-${String(month).padStart(2, '0')}-01T00:00:00`;
		const nextMonth = month === 12 ? 1 : month + 1;
		const nextYear = month === 12 ? currentYear + 1 : currentYear;
		const monthEnd = `${nextYear}-${String(nextMonth).padStart(2, '0')}-01T00:00:00`;

		const placeholders = projectIds.map(() => '?').join(',');
		const result = await db.execute({
			sql: `SELECT
                    COALESCE(SUM(active_seconds), 0) as total_seconds,
                    COUNT(*) as session_count
                  FROM sessions
                  WHERE project_id IN (${placeholders})
                  AND started_at >= ? AND started_at < ?
                  AND status != 'active'`,
			args: [...projectIds, monthStart, monthEnd]
		});

		monthlyStats.push({
			month,
			hours: Number(result.rows[0]?.total_seconds ?? 0) / 3600,
			sessions: Number(result.rows[0]?.session_count ?? 0)
		});
	}

	// Get yearly totals
	let yearlyHours = 0;
	let yearlySessions = 0;

	if (projectIds.length > 0) {
		const placeholders = projectIds.map(() => '?').join(',');
		const yearlyResult = await db.execute({
			sql: `SELECT
                    COALESCE(SUM(active_seconds), 0) as total_seconds,
                    COUNT(*) as session_count
                  FROM sessions
                  WHERE project_id IN (${placeholders})
                  AND started_at >= ?
                  AND status != 'active'`,
			args: [...projectIds, `${currentYear}-01-01T00:00:00`]
		});
		yearlyHours = Number(yearlyResult.rows[0]?.total_seconds ?? 0) / 3600;
		yearlySessions = Number(yearlyResult.rows[0]?.session_count ?? 0);
	}

	const contractHours = contract?.total_hours ?? 0;
	const monthlyHours = contract?.monthly_hours ?? 0;
	const carriedOver = contract?.carried_over ?? 0;

	// 計算月度額度相關數據
	const currentMonth = new Date().getMonth() + 1; // 1-12
	const currentMonthHours = monthlyStats[currentMonth - 1]?.hours ?? 0;

	// 到當月為止的累積總額度 = (當前月份 × 月額度) + 結轉
	const accumulatedQuota = monthlyHours > 0 ? currentMonth * monthlyHours + carriedOver : 0;

	// 累積可用額度 = 累積總額度 - 年度已使用
	const availableQuota = Math.max(0, accumulatedQuota - yearlyHours);

	// 當月額度使用率（如果有月額度的話）
	const monthlyUsagePercent =
		monthlyHours > 0 ? (currentMonthHours / monthlyHours) * 100 : 0;

	// 年度額度使用率
	const remainingHours = Math.max(0, contractHours - yearlyHours);
	const usagePercent = contractHours > 0 ? (yearlyHours / contractHours) * 100 : 0;

	// Filter to only months with data for the report list
	const monthsWithData = monthlyStats.filter((m) => m.hours > 0 || m.sessions > 0);

	return {
		client,
		projects,
		contract,
		monthlyStats,
		monthsWithData,
		yearlyHours,
		yearlySessions,
		contractHours,
		remainingHours,
		usagePercent,
		currentYear,
		// 新增月度額度相關數據
		monthlyHours,
		carriedOver,
		currentMonth,
		currentMonthHours,
		accumulatedQuota,
		availableQuota,
		monthlyUsagePercent
	};
};
