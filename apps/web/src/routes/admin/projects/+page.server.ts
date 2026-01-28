import { fail } from '@sveltejs/kit';
import { superValidate, message } from 'sveltekit-superforms';
import { zod4 } from 'sveltekit-superforms/adapters';
import { z } from 'zod/v4';
import type { Actions, PageServerLoad } from './$types';
import {
	getAllProjects,
	getProjectById,
	updateProject,
	setProjectIgnored,
	getClients
} from '$lib/server/db';

// Schema for project form
const projectSchema = z.object({
	id: z.number(),
	display_name: z.string().max(100, '名稱過長').nullable(),
	client_id: z.number().nullable(),
	work_item_pattern: z.string().max(200, '模式過長').nullable(),
	work_item_example: z.string().max(50).optional() // For UI input, not stored
});

const toggleIgnoreSchema = z.object({
	id: z.number(),
	ignored: z.boolean()
});

/**
 * Infer regex pattern from example work item
 * e.g., "BRU-321" -> "BRU-\\d+"
 * e.g., "PROJ-1234" -> "PROJ-\\d+"
 */
function inferWorkItemPattern(example: string): string | null {
	if (!example.trim()) return null;

	// Try common patterns like PREFIX-NUMBER
	const prefixNumberMatch = example.match(/^([A-Z]+)-(\d+)$/i);
	if (prefixNumberMatch) {
		return `${prefixNumberMatch[1].toUpperCase()}-\\d+`;
	}

	// Try NUMBER only
	if (/^\d+$/.test(example)) {
		return '\\d+';
	}

	// Try PREFIX NUMBER (no dash)
	const prefixSpaceMatch = example.match(/^([A-Z]+)(\d+)$/i);
	if (prefixSpaceMatch) {
		return `${prefixSpaceMatch[1].toUpperCase()}\\d+`;
	}

	// Return the example as-is if no pattern detected
	return example;
}

export const load: PageServerLoad = async ({ url }) => {
	const showIgnored = url.searchParams.get('showIgnored') === 'true';
	const [projects, clients] = await Promise.all([getAllProjects(showIgnored), getClients()]);
	const form = await superValidate(zod4(projectSchema));
	const toggleIgnoreForm = await superValidate(zod4(toggleIgnoreSchema));

	return { projects, clients, form, toggleIgnoreForm, showIgnored };
};

export const actions: Actions = {
	update: async ({ request }) => {
		const form = await superValidate(request, zod4(projectSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		const { id, display_name, client_id, work_item_example } = form.data;

		// Infer pattern from example if provided
		let work_item_pattern = form.data.work_item_pattern;
		if (work_item_example) {
			work_item_pattern = inferWorkItemPattern(work_item_example);
		}

		try {
			const updated = await updateProject(id, {
				display_name,
				client_id,
				work_item_pattern
			});

			if (!updated) {
				return message(form, { type: 'error', text: '找不到此專案' }, { status: 404 });
			}

			return message(form, { type: 'success', text: '專案已更新' });
		} catch (err) {
			console.error('Failed to update project:', err);
			return message(form, { type: 'error', text: '更新失敗，請稍後再試' }, { status: 500 });
		}
	},

	toggleIgnore: async ({ request }) => {
		const formData = await request.formData();
		const id = Number(formData.get('id'));
		const ignored = formData.get('ignored') === 'true';

		if (!id || isNaN(id)) {
			return fail(400, { error: '無效的專案 ID' });
		}

		try {
			const success = await setProjectIgnored(id, ignored);
			if (!success) {
				return fail(404, { error: '找不到此專案' });
			}
			return { success: true };
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : '操作失敗';
			return fail(400, { error: errorMessage });
		}
	}
};
