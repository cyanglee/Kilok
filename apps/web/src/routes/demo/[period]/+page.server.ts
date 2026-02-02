import type { PageServerLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ params }) => {
	const { period } = params;

	// Validate period format
	const periodMatch = period.match(/^(\d{4})-(\d{2})$/);
	if (!periodMatch) {
		throw error(404, '無效的日期格式');
	}

	const year = parseInt(periodMatch[1]);
	const month = parseInt(periodMatch[2]);
	const currentYear = new Date().getFullYear();

	// Calculate prev/next periods
	const prevMonth = month === 1 ? 12 : month - 1;
	const prevYear = month === 1 ? year - 1 : year;
	const nextMonth = month === 12 ? 1 : month + 1;
	const nextYear = month === 12 ? year + 1 : year;
	const prevPeriod = `${prevYear}-${String(prevMonth).padStart(2, '0')}`;
	const nextPeriod = `${nextYear}-${String(nextMonth).padStart(2, '0')}`;

	// Mock work items based on period
	const mockWorkItems: Record<string, Array<{
		identifier: string;
		title: string;
		description: string | null;
		completedDate: string | null;
		billableHours: number;
	}>> = {
		[`${currentYear}-01`]: [
			{
				identifier: 'feature/user-auth',
				title: '使用者認證系統',
				description: `2026/01/15 | 登入頁面 | 2.5h | 實作 OAuth 整合\n2026/01/18 | 權限管理 | 1.5h | 新增角色權限設定`,
				completedDate: '2026-01-20',
				billableHours: 4.0
			},
			{
				identifier: 'fix/dashboard-bug',
				title: '修復儀表板顯示問題',
				description: '修正在 Safari 上的 CSS 相容性問題',
				completedDate: '2026-01-25',
				billableHours: 1.5
			},
			{
				identifier: 'feature/api-integration',
				title: 'API 串接優化',
				description: `2026/01/28 | 快取機制 | 1.5h | 實作 Redis 快取\n2026/01/30 | 錯誤處理 | 1.5h | 改善 API 錯誤回傳格式`,
				completedDate: '2026-01-30',
				billableHours: 3.0
			}
		],
		[`${currentYear}-02`]: [
			{
				identifier: 'feature/report-export',
				title: '報表匯出功能',
				description: `2026/02/05 | PDF 匯出 | 3h | 實作 PDF 報表產生\n2026/02/08 | Excel 匯出 | 2h | 新增 XLSX 格式支援`,
				completedDate: '2026-02-10',
				billableHours: 5.0
			},
			{
				identifier: 'feature/notification',
				title: '通知系統',
				description: '實作即時通知功能，支援 Email 和 Push Notification',
				completedDate: '2026-02-15',
				billableHours: 4.0
			},
			{
				identifier: 'chore/performance',
				title: '效能優化',
				description: `2026/02/20 | 資料庫查詢 | 1.5h | 優化慢查詢\n2026/02/22 | 前端載入 | 1.5h | 實作 lazy loading`,
				completedDate: '2026-02-22',
				billableHours: 3.0
			}
		]
	};

	const workItems = mockWorkItems[period] || [];
	const completedBillableHours = workItems.reduce((sum, item) => sum + item.billableHours, 0);

	return {
		clientName: 'Acme Corp',
		year,
		month,
		period,
		prevPeriod,
		nextPeriod,
		monthlyHours: 8,
		completedBillableHours,
		completedWorkItems: workItems,
		isDemo: true
	};
};
