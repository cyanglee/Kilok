import {
	getClientByShareToken,
	getProjectsByClientId,
	getSessionsInMonth
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

	// Calculate monthly stats (last 12 months)
	const now = new Date();
	const monthlyStats: { period: string; hours: number; billableHours: number }[] = [];

	for (let i = 0; i < 12; i++) {
		const date = new Date(now.getFullYear(), now.getMonth() - i, 1);
		const year = date.getFullYear();
		const month = date.getMonth() + 1;
		const period = `${year}-${String(month).padStart(2, '0')}`;

		const sessions = await getSessionsInMonth(projectIds, year, month);
		const totalSeconds = sessions.reduce((sum, s) => sum + (s.active_seconds ?? 0), 0);

		if (totalSeconds > 0) {
			monthlyStats.push({
				period,
				hours: totalSeconds / 3600,
				billableHours: calculateBillableHours(totalSeconds)
			});
		}
	}

	return {
		token: params.token,
		clientName: client.name,
		monthlyStats
	};
};
