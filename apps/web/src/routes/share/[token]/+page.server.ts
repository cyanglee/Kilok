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

		// Get billable hours from completed work items
		const completedWorkItems = await getCompletedWorkItemsInMonth(projectIds, year, month);
		const workItemBillableHours = completedWorkItems.reduce(
			(sum, wi) => sum + (wi.billable_hours ?? 0),
			0
		);

		// Also count session-based completed work items billable hours
		// For simplicity, use work item billable hours if available, otherwise calculate
		const billableHours = workItemBillableHours > 0
			? workItemBillableHours
			: calculateBillableHours(totalSeconds);

		if (totalSeconds > 0 || workItemBillableHours > 0) {
			monthlyStats.push({
				period,
				year,
				month,
				hours: totalSeconds / 3600,
				billableHours
			});

			// Sum up yearly billable hours
			if (year === currentYear) {
				yearlyBillableHours += billableHours;
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
		usagePercent
	};
};
