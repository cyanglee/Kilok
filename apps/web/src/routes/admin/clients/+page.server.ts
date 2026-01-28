import { fail } from '@sveltejs/kit';
import { superValidate, message } from 'sveltekit-superforms';
import { zod4 } from 'sveltekit-superforms/adapters';
import { z } from 'zod/v4';
import type { Actions, PageServerLoad } from './$types';
import {
	getClients,
	getClientById,
	createClient,
	updateClient,
	deleteClient,
	clientSlugExists,
	getProjectsByClientId
} from '$lib/server/db';
import { generateSlug } from '$lib/slug';

// Schema for client form
const clientSchema = z.object({
	id: z.number().optional(),
	name: z.string().min(1, '請輸入客戶名稱').max(100, '名稱過長'),
	slug: z.string().min(1, '請輸入 Slug').max(50, 'Slug 過長').regex(/^[\w\u4e00-\u9fff-]+$/, 'Slug 只能包含字母、數字、中文和連字號')
});

const deleteSchema = z.object({
	id: z.number()
});

export const load: PageServerLoad = async () => {
	const clients = await getClients();
	const form = await superValidate(zod4(clientSchema));
	const deleteForm = await superValidate(zod4(deleteSchema));

	// Get project counts for each client
	const clientsWithCounts = await Promise.all(
		clients.map(async (client) => {
			const projects = await getProjectsByClientId(client.id);
			return { ...client, projectCount: projects.length };
		})
	);

	return { clients: clientsWithCounts, form, deleteForm };
};

export const actions: Actions = {
	create: async ({ request }) => {
		const form = await superValidate(request, zod4(clientSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		const { name, slug } = form.data;

		// Check if slug already exists
		if (await clientSlugExists(slug)) {
			return message(form, { type: 'error', text: '此 Slug 已被使用' }, { status: 400 });
		}

		try {
			await createClient(name, slug);
			return message(form, { type: 'success', text: '客戶已建立' });
		} catch (err) {
			console.error('Failed to create client:', err);
			return message(form, { type: 'error', text: '建立失敗，請稍後再試' }, { status: 500 });
		}
	},

	update: async ({ request }) => {
		const form = await superValidate(request, zod4(clientSchema));

		if (!form.valid || !form.data.id) {
			return fail(400, { form });
		}

		const { id, name, slug } = form.data;

		// Check if slug already exists (excluding current client)
		if (await clientSlugExists(slug, id)) {
			return message(form, { type: 'error', text: '此 Slug 已被使用' }, { status: 400 });
		}

		try {
			const updated = await updateClient(id, name, slug);
			if (!updated) {
				return message(form, { type: 'error', text: '找不到此客戶' }, { status: 404 });
			}
			return message(form, { type: 'success', text: '客戶已更新' });
		} catch (err) {
			console.error('Failed to update client:', err);
			return message(form, { type: 'error', text: '更新失敗，請稍後再試' }, { status: 500 });
		}
	},

	delete: async ({ request }) => {
		const form = await superValidate(request, zod4(deleteSchema));

		if (!form.valid) {
			return fail(400, { deleteForm: form });
		}

		try {
			await deleteClient(form.data.id);
			return { deleteForm: form, success: true };
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : '刪除失敗';
			return message(form, { type: 'error', text: errorMessage }, { status: 400 });
		}
	},

	generateSlug: async ({ request }) => {
		const formData = await request.formData();
		const name = formData.get('name');

		if (typeof name !== 'string') {
			return fail(400, { slug: '' });
		}

		return { slug: generateSlug(name) };
	}
};
