/**
 * Format hours as "Xh Ym" or "Xh"
 */
export function formatHours(hours: number): string {
	const h = Math.floor(hours);
	const m = Math.round((hours - h) * 60);
	return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

/**
 * Format billable hours (always show .5 if applicable)
 */
export function formatBillableHours(hours: number): string {
	if (hours % 1 === 0.5) {
		return `${hours}h`;
	}
	return `${Math.floor(hours)}h`;
}

/**
 * Format seconds as hours
 */
export function formatSeconds(seconds: number): string {
	return formatHours(seconds / 3600);
}

/**
 * Format date string (YYYY-MM-DD) to MM/DD
 */
export function formatDate(dateStr: string): string {
	const parts = dateStr.split('-');
	if (parts.length === 3) {
		return `${parts[1]}/${parts[2]}`;
	}
	return dateStr;
}

/**
 * Format period string (YYYY-MM) to "YYYY 年 X月"
 */
export function formatPeriod(period: string): string {
	const [year, month] = period.split('-');
	return `${year} 年 ${monthNames[parseInt(month, 10) - 1]}`;
}

/**
 * Month names in Chinese
 */
export const monthNames = [
	'一月', '二月', '三月', '四月', '五月', '六月',
	'七月', '八月', '九月', '十月', '十一月', '十二月'
];
