import { fail } from '@sveltejs/kit';
import { superValidate, message } from 'sveltekit-superforms';
import { zod4 } from 'sveltekit-superforms/adapters';
import { z } from 'zod/v4';
import type { Actions, PageServerLoad } from './$types';
import {
	getAllContracts,
	getContractById,
	createContract,
	updateContract,
	deleteContract,
	contractExists,
	getClients
} from '$lib/server/db';

// Schema for contract form
const contractSchema = z.object({
	id: z.number().optional(),
	client_id: z.number({ message: '請選擇客戶' }),
	year: z
		.number({ message: '請輸入年份' })
		.min(2020, '年份不能小於 2020')
		.max(2100, '年份不能大於 2100'),
	total_hours: z.number({ message: '請輸入合約時數' }).min(0, '時數不能為負數'),
	carried_over: z.number().min(0, '結轉時數不能為負數').default(0)
});

const deleteSchema = z.object({
	id: z.number()
});

export const load: PageServerLoad = async () => {
	const [contracts, clients] = await Promise.all([getAllContracts(), getClients()]);
	const form = await superValidate(zod4(contractSchema));
	const deleteForm = await superValidate(zod4(deleteSchema));

	// Set default year to current year
	form.data.year = new Date().getFullYear();
	form.data.total_hours = 0;
	form.data.carried_over = 0;

	return { contracts, clients, form, deleteForm };
};

export const actions: Actions = {
	create: async ({ request }) => {
		const form = await superValidate(request, zod4(contractSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		const { client_id, year, total_hours, carried_over } = form.data;

		// Check if contract already exists for this client and year
		if (await contractExists(client_id, year)) {
			return message(form, { type: 'error', text: '此客戶在該年度已有合約' }, { status: 400 });
		}

		try {
			await createContract(client_id, year, total_hours, carried_over);
			return message(form, { type: 'success', text: '合約已建立' });
		} catch (err) {
			console.error('Failed to create contract:', err);
			return message(form, { type: 'error', text: '建立失敗，請稍後再試' }, { status: 500 });
		}
	},

	update: async ({ request }) => {
		const form = await superValidate(request, zod4(contractSchema));

		if (!form.valid || !form.data.id) {
			return fail(400, { form });
		}

		const { id, client_id, year, total_hours, carried_over } = form.data;

		// Check if another contract exists for this client and year
		if (await contractExists(client_id, year, id)) {
			return message(form, { type: 'error', text: '此客戶在該年度已有其他合約' }, { status: 400 });
		}

		try {
			const updated = await updateContract(id, { year, total_hours, carried_over });

			if (!updated) {
				return message(form, { type: 'error', text: '找不到此合約' }, { status: 404 });
			}

			return message(form, { type: 'success', text: '合約已更新' });
		} catch (err) {
			console.error('Failed to update contract:', err);
			return message(form, { type: 'error', text: '更新失敗，請稍後再試' }, { status: 500 });
		}
	},

	delete: async ({ request }) => {
		const form = await superValidate(request, zod4(deleteSchema));

		if (!form.valid) {
			return fail(400, { deleteForm: form });
		}

		try {
			const success = await deleteContract(form.data.id);
			if (!success) {
				return message(form, { type: 'error', text: '找不到此合約' }, { status: 404 });
			}
			return { deleteForm: form, success: true };
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : '刪除失敗';
			return message(form, { type: 'error', text: errorMessage }, { status: 400 });
		}
	}
};
