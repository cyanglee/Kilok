import type { PageServerLoad } from './$types';

// Mock data for demo
export const load: PageServerLoad = async () => {
	const currentYear = new Date().getFullYear();
	const currentMonth = new Date().getMonth() + 1;

	// Generate realistic monthly stats
	const monthlyStats = [
		{ period: `${currentYear}-01`, month: 1, year: currentYear, billableHours: 8.5, rawSeconds: 25500 },
		{ period: `${currentYear}-02`, month: 2, year: currentYear, billableHours: 12.0, rawSeconds: 36000 },
		{ period: `${currentYear - 1}-11`, month: 11, year: currentYear - 1, billableHours: 6.5, rawSeconds: 19500 },
		{ period: `${currentYear - 1}-12`, month: 12, year: currentYear - 1, billableHours: 10.0, rawSeconds: 30000 }
	].filter(s => s.year === currentYear || (s.year === currentYear - 1 && s.month >= 11));

	// Only current year stats for calculation
	const currentYearStats = monthlyStats.filter(s => s.year === currentYear);
	const yearlyBillableHours = currentYearStats.reduce((sum, s) => sum + s.billableHours, 0);

	const monthlyHours = 8; // 8 hours per month contract
	const carriedOver = 4; // 4 hours carried from last year
	const yearlyQuota = monthlyHours * 12 + carriedOver;
	const remainingHours = yearlyQuota - yearlyBillableHours;
	const usagePercent = (yearlyBillableHours / yearlyQuota) * 100;

	return {
		clientName: 'Acme Corp',
		currentYear,
		currentMonth,
		hasContract: true,
		monthlyHours,
		carriedOver,
		yearlyQuota,
		yearlyBillableHours,
		remainingHours,
		usagePercent,
		monthlyStats: currentYearStats.sort((a, b) => b.period.localeCompare(a.period)),
		isDemo: true
	};
};
