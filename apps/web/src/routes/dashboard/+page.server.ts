import { redirect } from '@sveltejs/kit';
import {
	getClients,
	getProjectsByClientId,
	getContractByClientAndYear,
	getUnassignedProjects,
	db
} from '$lib/server/db';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies }) => {
	// Authentication check
	const session = cookies.get('admin_session');
	if (session !== 'authenticated') {
		throw redirect(303, '/admin/login');
	}

	const clients = await getClients();
	const unassignedProjects = await getUnassignedProjects();
	const currentYear = new Date().getFullYear();
	const currentMonth = new Date().getMonth() + 1;

	// Get stats for each client
	const clientStats = await Promise.all(
		clients.map(async (client) => {
			const projects = await getProjectsByClientId(client.id);
			const contract = await getContractByClientAndYear(client.id, currentYear);

			// Get total hours this year for this client
			const projectIds = projects.map((p) => p.id);
			let yearlyHours = 0;
			let monthlyHours = 0;

			if (projectIds.length > 0) {
				const placeholders = projectIds.map(() => '?').join(',');

				// Yearly hours
				const yearlyResult = await db.execute({
					sql: `SELECT COALESCE(SUM(active_seconds), 0) as total
                          FROM sessions
                          WHERE project_id IN (${placeholders})
                          AND started_at >= ?
                          AND status != 'active'`,
					args: [...projectIds, `${currentYear}-01-01T00:00:00`]
				});
				yearlyHours = Number(yearlyResult.rows[0]?.total ?? 0) / 3600;

				// Monthly hours
				const monthStart = `${currentYear}-${String(currentMonth).padStart(2, '0')}-01T00:00:00`;
				const nextMonth = currentMonth === 12 ? 1 : currentMonth + 1;
				const nextYear = currentMonth === 12 ? currentYear + 1 : currentYear;
				const monthEnd = `${nextYear}-${String(nextMonth).padStart(2, '0')}-01T00:00:00`;

				const monthlyResult = await db.execute({
					sql: `SELECT COALESCE(SUM(active_seconds), 0) as total
                          FROM sessions
                          WHERE project_id IN (${placeholders})
                          AND started_at >= ? AND started_at < ?
                          AND status != 'active'`,
					args: [...projectIds, monthStart, monthEnd]
				});
				monthlyHours = Number(monthlyResult.rows[0]?.total ?? 0) / 3600;
			}

			const contractHours = contract?.total_hours ?? 0;
			const usagePercent = contractHours > 0 ? (yearlyHours / contractHours) * 100 : 0;

			return {
				client,
				projects,
				contract,
				yearlyHours,
				monthlyHours,
				contractHours,
				usagePercent,
				remainingHours: Math.max(0, contractHours - yearlyHours)
			};
		})
	);

	// Calculate unassigned projects stats
	const unassignedStats = await Promise.all(
		unassignedProjects.map(async (project) => {
			let yearlyHours = 0;
			let monthlyHours = 0;

			// Yearly hours
			const yearlyResult = await db.execute({
				sql: `SELECT COALESCE(SUM(active_seconds), 0) as total
					  FROM sessions
					  WHERE project_id = ?
					  AND started_at >= ?
					  AND status != 'active'`,
				args: [project.id, `${currentYear}-01-01T00:00:00`]
			});
			yearlyHours = Number(yearlyResult.rows[0]?.total ?? 0) / 3600;

			// Monthly hours
			const monthStart = `${currentYear}-${String(currentMonth).padStart(2, '0')}-01T00:00:00`;
			const nextMonth = currentMonth === 12 ? 1 : currentMonth + 1;
			const nextYear = currentMonth === 12 ? currentYear + 1 : currentYear;
			const monthEnd = `${nextYear}-${String(nextMonth).padStart(2, '0')}-01T00:00:00`;

			const monthlyResult = await db.execute({
				sql: `SELECT COALESCE(SUM(active_seconds), 0) as total
					  FROM sessions
					  WHERE project_id = ?
					  AND started_at >= ? AND started_at < ?
					  AND status != 'active'`,
				args: [project.id, monthStart, monthEnd]
			});
			monthlyHours = Number(monthlyResult.rows[0]?.total ?? 0) / 3600;

			return {
				project,
				yearlyHours,
				monthlyHours
			};
		})
	);

	// Calculate totals (including unassigned projects)
	const clientMonthlyHours = clientStats.reduce((sum, s) => sum + s.monthlyHours, 0);
	const clientYearlyHours = clientStats.reduce((sum, s) => sum + s.yearlyHours, 0);
	const unassignedMonthlyHours = unassignedStats.reduce((sum, s) => sum + s.monthlyHours, 0);
	const unassignedYearlyHours = unassignedStats.reduce((sum, s) => sum + s.yearlyHours, 0);

	const totalMonthlyHours = clientMonthlyHours + unassignedMonthlyHours;
	const totalYearlyHours = clientYearlyHours + unassignedYearlyHours;
	const activeProjects =
		clientStats.reduce((sum, s) => sum + s.projects.length, 0) + unassignedProjects.length;

	return {
		clientStats,
		unassignedStats,
		totalMonthlyHours,
		totalYearlyHours,
		activeProjects,
		currentYear,
		currentMonth
	};
};
