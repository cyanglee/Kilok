import {
	getClientByShareToken,
	getProjectsByClientId,
	getSessionsInMonth,
	getContractByClientAndYear,
	getCompletedWorkItemsInMonth,
	db
} from '$lib/server/db';
import { calculateBillableHours } from '$lib/billable';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params }) => {
	// Look up client by share token
	const client = await getClientByShareToken(params.token);
	if (!client) {
		throw error(404, '分享連結不存在');
	}

	// Get available months for this client
	const projects = await getProjectsByClientId(client.id);
	const projectIds = projects.map((p) => p.id);

	const currentYear = new Date().getFullYear();
	const currentMonth = new Date().getMonth() + 1;
	const contract = await getContractByClientAndYear(client.id, currentYear);

	// Calculate monthly stats (last 12 months) with billable hours from work items
	const now = new Date();
	const monthlyStats: { period: string; year: number; month: number; hours: number; billableHours: number }[] = [];

	let yearlyBillableHours = 0;

	for (let i = 0; i < 12; i++) {
		const date = new Date(now.getFullYear(), now.getMonth() - i, 1);
		const year = date.getFullYear();
		const month = date.getMonth() + 1;
		const period = `${year}-${String(month).padStart(2, '0')}`;

		const sessions = await getSessionsInMonth(projectIds, year, month);
		const totalSeconds = sessions.reduce((sum, s) => sum + (s.active_seconds ?? 0), 0);

		// Get billable hours from completed work items only
		// Client-facing share page should only show completed/reported items
		const completedWorkItems = await getCompletedWorkItemsInMonth(projectIds, year, month);
		const workItemBillableHours = completedWorkItems.reduce(
			(sum, wi) => sum + (wi.billable_hours ?? 0),
			0
		);

		// Only show months with completed work items on client share page
		if (workItemBillableHours > 0) {
			monthlyStats.push({
				period,
				year,
				month,
				hours: totalSeconds / 3600,
				billableHours: workItemBillableHours
			});

			// Sum up yearly billable hours
			if (year === currentYear) {
				yearlyBillableHours += workItemBillableHours;
			}
		}
	}

	// Contract calculations
	const contractHours = contract?.total_hours ?? 0;
	const monthlyHours = contract?.monthly_hours ?? 0;
	const carriedOver = contract?.carried_over ?? 0;

	// 年度總額度 = (12 × 月額度) + 結轉，或使用 total_hours
	const yearlyQuota = monthlyHours > 0
		? 12 * monthlyHours + carriedOver
		: contractHours;

	// 到當月累積額度 = (當前月份 × 月額度) + 結轉
	const accumulatedQuota = monthlyHours > 0
		? currentMonth * monthlyHours + carriedOver
		: contractHours;

	// 剩餘額度（相對於年度總額度）
	const remainingHours = Math.max(0, yearlyQuota - yearlyBillableHours);

	// 使用率（相對於年度總額度）
	const usagePercent = yearlyQuota > 0
		? (yearlyBillableHours / yearlyQuota) * 100
		: 0;

	// 計算每月累積額度明細（只在有月額度時）
	const monthlyQuotaBreakdown: Array<{
		month: number;
		quota: number;        // 當月新增額度
		used: number;         // 當月使用
		startBalance: number; // 月初餘額
		endBalance: number;   // 月底餘額
	}> = [];

	if (monthlyHours > 0) {
		let runningBalance = carriedOver; // 從結轉開始

		for (let month = 1; month <= 12; month++) {
			const startBalance = runningBalance;
			const quota = monthlyHours;

			// 找出該月的使用量
			const monthStat = monthlyStats.find(s => s.year === currentYear && s.month === month);
			const used = monthStat?.billableHours ?? 0;

			// 月底餘額 = 月初 + 當月額度 - 當月使用
			const endBalance = startBalance + quota - used;

			monthlyQuotaBreakdown.push({
				month,
				quota,
				used,
				startBalance,
				endBalance
			});

			// 下個月的月初餘額
			runningBalance = endBalance;
		}
	}

	return {
		token: params.token,
		clientName: client.name,
		monthlyStats,
		currentYear,
		currentMonth,
		// Contract info
		hasContract: contractHours > 0 || monthlyHours > 0,
		contractHours,
		monthlyHours,
		carriedOver,
		yearlyQuota,
		accumulatedQuota,
		yearlyBillableHours,
		remainingHours,
		usagePercent,
		monthlyQuotaBreakdown
	};
};
