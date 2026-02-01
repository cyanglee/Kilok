import { createClient as createDbClient } from '@libsql/client';
import { TURSO_DATABASE_URL, TURSO_AUTH_TOKEN } from '$env/static/private';

export const db = createDbClient({
	url: TURSO_DATABASE_URL,
	authToken: TURSO_AUTH_TOKEN
});

// Schema migrations - run once on startup
let schemaInitialized = false;
async function ensureSchema() {
	if (schemaInitialized) return;
	try {
		// Add monthly_hours column to contracts table
		await db.execute('ALTER TABLE contracts ADD COLUMN monthly_hours REAL DEFAULT 0');
	} catch {
		// Column already exists, ignore
	}
	try {
		// Add share_token column to clients table (simplified: one token per client)
		await db.execute('ALTER TABLE clients ADD COLUMN share_token TEXT');
	} catch {
		// Column already exists, ignore
	}
	try {
		// Create index for share_token lookups
		await db.execute('CREATE UNIQUE INDEX IF NOT EXISTS idx_clients_share_token ON clients(share_token)');
	} catch {
		// Index already exists, ignore
	}
	try {
		// Generate tokens for existing clients that don't have one
		const clientsWithoutToken = await db.execute(
			'SELECT id FROM clients WHERE share_token IS NULL'
		);
		for (const row of clientsWithoutToken.rows) {
			const client = row as unknown as { id: number };
			const token = crypto.randomUUID();
			await db.execute({
				sql: 'UPDATE clients SET share_token = ? WHERE id = ?',
				args: [token, client.id]
			});
		}
	} catch (e) {
		console.error('Failed to generate tokens for existing clients:', e);
	}
	try {
		// Add billable_hours column to work_items (nullable override for calculated value)
		await db.execute('ALTER TABLE work_items ADD COLUMN billable_hours REAL');
	} catch {
		// Column already exists, ignore
	}
	schemaInitialized = true;
}

// Initialize schema on module load
ensureSchema().catch(console.error);

// Types matching Rust CLI models
export interface Client {
	id: number;
	slug: string;
	name: string;
	share_token: string | null;
	created_at: string;
}

export interface Contract {
	id: number;
	client_id: number;
	year: number;
	total_hours: number;
	monthly_hours: number;
	carried_over: number;
	created_at: string;
}

export interface Project {
	id: number;
	path: string;
	git_remote: string | null;
	display_name: string | null;
	work_item_pattern: string | null;
	client_id: number | null;
	ignored: boolean;
	created_at: string;
}

export interface Session {
	id: number;
	project_id: number;
	branch: string;
	work_item: string | null;
	start_commit: string | null;
	end_commit: string | null;
	started_at: string;
	ended_at: string | null;
	active_seconds: number | null;
	status: 'active' | 'completed' | 'abandoned';
}

export interface Commit {
	id: number;
	session_id: number;
	hash: string;
	message: string | null;
	committed_at: string | null;
}

export interface WorkItem {
	id: number;
	project_id: number;
	identifier: string;
	title: string | null;
	description: string | null;
	time_adjustment_seconds: number;
	completed_date: string | null;
	billable_hours: number | null; // Override for calculated billable hours
	created_at: string;
	updated_at: string | null;
}


// Query helpers
export async function getClients(): Promise<Client[]> {
	const result = await db.execute('SELECT * FROM clients ORDER BY name');
	return result.rows as unknown as Client[];
}

export async function getClientBySlug(slug: string): Promise<Client | null> {
	const result = await db.execute({
		sql: 'SELECT * FROM clients WHERE slug = ?',
		args: [slug]
	});
	return (result.rows[0] as unknown as Client) ?? null;
}

export async function getProjectsByClientId(clientId: number): Promise<Project[]> {
	const result = await db.execute({
		sql: 'SELECT * FROM projects WHERE client_id = ? ORDER BY display_name',
		args: [clientId]
	});
	return result.rows as unknown as Project[];
}

export async function getSessionsInMonth(
	projectIds: number[],
	year: number,
	month: number
): Promise<Session[]> {
	if (projectIds.length === 0) return [];

	const startDate = `${year}-${String(month).padStart(2, '0')}-01T00:00:00`;
	const endMonth = month === 12 ? 1 : month + 1;
	const endYear = month === 12 ? year + 1 : year;
	const endDate = `${endYear}-${String(endMonth).padStart(2, '0')}-01T00:00:00`;

	const placeholders = projectIds.map(() => '?').join(',');
	const result = await db.execute({
		sql: `SELECT * FROM sessions
			  WHERE project_id IN (${placeholders})
			  AND started_at >= ? AND started_at < ?
			  AND status != 'active'
			  ORDER BY started_at`,
		args: [...projectIds, startDate, endDate]
	});
	return result.rows as unknown as Session[];
}

export async function getCommitsBySessionIds(sessionIds: number[]): Promise<Commit[]> {
	if (sessionIds.length === 0) return [];

	const placeholders = sessionIds.map(() => '?').join(',');
	const result = await db.execute({
		sql: `SELECT * FROM commits WHERE session_id IN (${placeholders}) ORDER BY committed_at`,
		args: sessionIds
	});
	return result.rows as unknown as Commit[];
}

export async function getContractByClientAndYear(
	clientId: number,
	year: number
): Promise<Contract | null> {
	const result = await db.execute({
		sql: 'SELECT * FROM contracts WHERE client_id = ? AND year = ?',
		args: [clientId, year]
	});
	return (result.rows[0] as unknown as Contract) ?? null;
}

// ========================================
// Admin CRUD Operations
// ========================================

// --- Clients ---

export async function getClientById(id: number): Promise<Client | null> {
	const result = await db.execute({
		sql: 'SELECT * FROM clients WHERE id = ?',
		args: [id]
	});
	return (result.rows[0] as unknown as Client) ?? null;
}

export async function createClient(name: string, slug: string): Promise<Client> {
	const shareToken = crypto.randomUUID();
	const result = await db.execute({
		sql: 'INSERT INTO clients (name, slug, share_token) VALUES (?, ?, ?) RETURNING *',
		args: [name, slug, shareToken]
	});
	return result.rows[0] as unknown as Client;
}

export async function getClientByShareToken(token: string): Promise<Client | null> {
	const result = await db.execute({
		sql: 'SELECT * FROM clients WHERE share_token = ?',
		args: [token]
	});
	return (result.rows[0] as unknown as Client) ?? null;
}

export async function regenerateClientShareToken(clientId: number): Promise<Client | null> {
	const newToken = crypto.randomUUID();
	const result = await db.execute({
		sql: 'UPDATE clients SET share_token = ? WHERE id = ? RETURNING *',
		args: [newToken, clientId]
	});
	return (result.rows[0] as unknown as Client) ?? null;
}

export async function updateClient(id: number, name: string, slug: string): Promise<Client | null> {
	const result = await db.execute({
		sql: 'UPDATE clients SET name = ?, slug = ? WHERE id = ? RETURNING *',
		args: [name, slug, id]
	});
	return (result.rows[0] as unknown as Client) ?? null;
}

export async function deleteClient(id: number): Promise<boolean> {
	// Check if client has projects
	const projectCheck = await db.execute({
		sql: 'SELECT COUNT(*) as count FROM projects WHERE client_id = ?',
		args: [id]
	});
	const projectCount = (projectCheck.rows[0] as unknown as { count: number }).count;
	if (projectCount > 0) {
		throw new Error('無法刪除：此客戶還有關聯的專案');
	}

	const result = await db.execute({
		sql: 'DELETE FROM clients WHERE id = ?',
		args: [id]
	});
	return result.rowsAffected > 0;
}

export async function clientSlugExists(slug: string, excludeId?: number): Promise<boolean> {
	const sql = excludeId
		? 'SELECT COUNT(*) as count FROM clients WHERE slug = ? AND id != ?'
		: 'SELECT COUNT(*) as count FROM clients WHERE slug = ?';
	const args = excludeId ? [slug, excludeId] : [slug];
	const result = await db.execute({ sql, args });
	return (result.rows[0] as unknown as { count: number }).count > 0;
}

// --- Projects ---

export async function getAllProjects(
	includeIgnored = false
): Promise<(Project & { client_name: string | null })[]> {
	const whereClause = includeIgnored ? '' : 'WHERE (p.ignored IS NULL OR p.ignored = 0)';
	const result = await db.execute(`
		SELECT p.*, c.name as client_name
		FROM projects p
		LEFT JOIN clients c ON p.client_id = c.id
		${whereClause}
		ORDER BY c.name, p.display_name
	`);
	// Convert ignored from 0/1 to boolean
	return result.rows.map((row) => {
		const project = row as unknown as Project & { client_name: string | null };
		project.ignored = Boolean(project.ignored);
		return project;
	});
}

export async function getProjectById(id: number): Promise<Project | null> {
	const result = await db.execute({
		sql: 'SELECT * FROM projects WHERE id = ?',
		args: [id]
	});
	return (result.rows[0] as unknown as Project) ?? null;
}

export async function updateProject(
	id: number,
	data: { display_name: string | null; client_id: number | null; work_item_pattern: string | null }
): Promise<Project | null> {
	const result = await db.execute({
		sql: 'UPDATE projects SET display_name = ?, client_id = ?, work_item_pattern = ? WHERE id = ? RETURNING *',
		args: [data.display_name, data.client_id, data.work_item_pattern, id]
	});
	return (result.rows[0] as unknown as Project) ?? null;
}

export async function setProjectIgnored(id: number, ignored: boolean): Promise<boolean> {
	// Soft delete: mark project as ignored instead of actually deleting
	// First ensure the ignored column exists (SQLite doesn't error on existing columns)
	try {
		await db.execute('ALTER TABLE projects ADD COLUMN ignored INTEGER DEFAULT 0');
	} catch {
		// Column already exists, ignore
	}

	const result = await db.execute({
		sql: 'UPDATE projects SET ignored = ? WHERE id = ?',
		args: [ignored ? 1 : 0, id]
	});
	return result.rowsAffected > 0;
}

// --- Contracts ---

export async function getAllContracts(): Promise<(Contract & { client_name: string })[]> {
	const result = await db.execute(`
		SELECT co.*, c.name as client_name
		FROM contracts co
		JOIN clients c ON co.client_id = c.id
		ORDER BY c.name, co.year DESC
	`);
	return result.rows as unknown as (Contract & { client_name: string })[];
}

export async function getContractById(id: number): Promise<Contract | null> {
	const result = await db.execute({
		sql: 'SELECT * FROM contracts WHERE id = ?',
		args: [id]
	});
	return (result.rows[0] as unknown as Contract) ?? null;
}

export async function createContract(
	clientId: number,
	year: number,
	totalHours: number,
	monthlyHours: number,
	carriedOver: number
): Promise<Contract> {
	const result = await db.execute({
		sql: 'INSERT INTO contracts (client_id, year, total_hours, monthly_hours, carried_over) VALUES (?, ?, ?, ?, ?) RETURNING *',
		args: [clientId, year, totalHours, monthlyHours, carriedOver]
	});
	return result.rows[0] as unknown as Contract;
}

export async function updateContract(
	id: number,
	data: { year: number; total_hours: number; monthly_hours: number; carried_over: number }
): Promise<Contract | null> {
	const result = await db.execute({
		sql: 'UPDATE contracts SET year = ?, total_hours = ?, monthly_hours = ?, carried_over = ? WHERE id = ? RETURNING *',
		args: [data.year, data.total_hours, data.monthly_hours, data.carried_over, id]
	});
	return (result.rows[0] as unknown as Contract) ?? null;
}

export async function deleteContract(id: number): Promise<boolean> {
	const result = await db.execute({
		sql: 'DELETE FROM contracts WHERE id = ?',
		args: [id]
	});
	return result.rowsAffected > 0;
}

export async function contractExists(
	clientId: number,
	year: number,
	excludeId?: number
): Promise<boolean> {
	const sql = excludeId
		? 'SELECT COUNT(*) as count FROM contracts WHERE client_id = ? AND year = ? AND id != ?'
		: 'SELECT COUNT(*) as count FROM contracts WHERE client_id = ? AND year = ?';
	const args = excludeId ? [clientId, year, excludeId] : [clientId, year];
	const result = await db.execute({ sql, args });
	return (result.rows[0] as unknown as { count: number }).count > 0;
}

// --- Unassigned Projects ---

export async function getUnassignedProjects(): Promise<Project[]> {
	const result = await db.execute(
		'SELECT * FROM projects WHERE client_id IS NULL ORDER BY display_name, path'
	);
	return result.rows as unknown as Project[];
}

// --- Work Items ---

export async function getWorkItemsByProjectIds(projectIds: number[]): Promise<WorkItem[]> {
	if (projectIds.length === 0) return [];

	const placeholders = projectIds.map(() => '?').join(',');
	const result = await db.execute({
		sql: `SELECT * FROM work_items WHERE project_id IN (${placeholders}) ORDER BY created_at DESC`,
		args: projectIds
	});
	return result.rows as unknown as WorkItem[];
}

export async function getWorkItemByIdentifier(
	projectId: number,
	identifier: string
): Promise<WorkItem | null> {
	const result = await db.execute({
		sql: 'SELECT * FROM work_items WHERE project_id = ? AND identifier = ?',
		args: [projectId, identifier]
	});
	return (result.rows[0] as unknown as WorkItem) ?? null;
}

export async function getCompletedWorkItemsInMonth(
	projectIds: number[],
	year: number,
	month: number
): Promise<WorkItem[]> {
	if (projectIds.length === 0) return [];

	const startDate = `${year}-${String(month).padStart(2, '0')}-01`;
	const endMonth = month === 12 ? 1 : month + 1;
	const endYear = month === 12 ? year + 1 : year;
	const endDate = `${endYear}-${String(endMonth).padStart(2, '0')}-01`;

	const placeholders = projectIds.map(() => '?').join(',');
	const result = await db.execute({
		sql: `SELECT * FROM work_items
			  WHERE project_id IN (${placeholders})
			  AND completed_date >= ? AND completed_date < ?
			  AND title IS NOT NULL
			  ORDER BY completed_date DESC`,
		args: [...projectIds, startDate, endDate]
	});
	return result.rows as unknown as WorkItem[];
}

export async function createWorkItem(
	projectId: number,
	identifier: string,
	data: {
		title: string;
		description?: string | null;
		completed_date?: string | null;
		billable_hours?: number | null;
	}
): Promise<WorkItem> {
	const result = await db.execute({
		sql: `INSERT INTO work_items (project_id, identifier, title, description, completed_date, billable_hours, time_adjustment_seconds)
			  VALUES (?, ?, ?, ?, ?, ?, 0) RETURNING *`,
		args: [
			projectId,
			identifier,
			data.title,
			data.description ?? null,
			data.completed_date ?? null,
			data.billable_hours ?? null
		]
	});
	return result.rows[0] as unknown as WorkItem;
}

export async function updateWorkItem(
	id: number,
	updates: {
		title?: string | null;
		description?: string | null;
		completed_date?: string | null;
		billable_hours?: number | null;
	}
): Promise<WorkItem | null> {
	const setClauses: string[] = ['updated_at = datetime("now")'];
	const args: (string | number | null)[] = [];

	if (updates.title !== undefined) {
		setClauses.push('title = ?');
		args.push(updates.title);
	}

	if (updates.description !== undefined) {
		setClauses.push('description = ?');
		args.push(updates.description);
	}

	if (updates.completed_date !== undefined) {
		setClauses.push('completed_date = ?');
		args.push(updates.completed_date);
	}

	if (updates.billable_hours !== undefined) {
		setClauses.push('billable_hours = ?');
		args.push(updates.billable_hours);
	}

	args.push(id);

	const result = await db.execute({
		sql: `UPDATE work_items SET ${setClauses.join(', ')} WHERE id = ? RETURNING *`,
		args
	});
	return (result.rows[0] as unknown as WorkItem) ?? null;
}

