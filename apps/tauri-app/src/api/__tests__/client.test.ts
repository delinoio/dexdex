import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { rpcCall } from '../client';

describe('rpcCall', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('calls fetch with correct URL and method', async () => {
    const mockData = { tasks: [] };
    vi.mocked(fetch).mockResolvedValue({
      ok: true,
      json: async () => mockData,
    } as Response);

    const result = await rpcCall('TaskService', 'List', { workspaceId: 'ws-1' });

    expect(fetch).toHaveBeenCalledWith(
      'http://localhost:3000/TaskService/List',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ workspaceId: 'ws-1' }),
      })
    );
    expect(result).toEqual(mockData);
  });

  it('throws an error when response is not ok', async () => {
    vi.mocked(fetch).mockResolvedValue({
      ok: false,
      status: 404,
      text: async () => 'Not Found',
    } as Response);

    await expect(rpcCall('TaskService', 'Get', { taskId: 'missing' })).rejects.toThrow(
      'TaskService/Get failed: 404 Not Found'
    );
  });

  it('throws a network error when fetch fails', async () => {
    vi.mocked(fetch).mockRejectedValue(new Error('Failed to fetch'));

    await expect(rpcCall('TaskService', 'List', {})).rejects.toThrow(
      'TaskService/List network error: Failed to fetch'
    );
  });

  it('serializes complex body correctly', async () => {
    vi.mocked(fetch).mockResolvedValue({
      ok: true,
      json: async () => ({ task: { id: 't1' } }),
    } as Response);

    await rpcCall('TaskService', 'Create', {
      workspaceId: 'ws-1',
      title: 'My Task',
      prompt: 'Do something',
    });

    expect(fetch).toHaveBeenCalledWith(
      'http://localhost:3000/TaskService/Create',
      expect.objectContaining({
        body: JSON.stringify({
          workspaceId: 'ws-1',
          title: 'My Task',
          prompt: 'Do something',
        }),
      })
    );
  });

  it('handles empty body for delete operations', async () => {
    vi.mocked(fetch).mockResolvedValue({
      ok: true,
      json: async () => ({}),
    } as Response);

    await rpcCall('TaskService', 'Delete', { taskId: 't1' });

    expect(fetch).toHaveBeenCalledWith(
      'http://localhost:3000/TaskService/Delete',
      expect.objectContaining({
        body: JSON.stringify({ taskId: 't1' }),
      })
    );
  });
});
