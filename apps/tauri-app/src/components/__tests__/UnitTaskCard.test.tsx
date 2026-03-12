import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { UnitTaskCard } from '../task/UnitTaskCard';
import type { UnitTask } from '@/api/types';

const mockTask: UnitTask = {
  id: 'task-1',
  workspaceId: 'ws-1',
  repositoryGroupId: 'rg-1',
  title: 'Fix the login bug',
  prompt: 'The login page crashes when users enter invalid credentials.',
  status: 'in_progress',
  actionTypes: [],
  prTrackingIds: [],
  generatedCommitCount: 0,
  createdAt: '2026-03-12T10:00:00Z',
  updatedAt: '2026-03-12T10:00:00Z',
};

describe('UnitTaskCard', () => {
  it('renders task title', () => {
    render(<UnitTaskCard task={mockTask} />);
    expect(screen.getByText('Fix the login bug')).toBeTruthy();
  });

  it('renders task status badge', () => {
    render(<UnitTaskCard task={mockTask} />);
    expect(screen.getByText('In Progress')).toBeTruthy();
  });

  it('renders task prompt', () => {
    render(<UnitTaskCard task={mockTask} />);
    expect(
      screen.getByText(
        'The login page crashes when users enter invalid credentials.'
      )
    ).toBeTruthy();
  });

  it('calls onClick when clicked', () => {
    const onClick = vi.fn();
    render(<UnitTaskCard task={mockTask} onClick={onClick} />);
    const button = screen.getByRole('button');
    fireEvent.click(button);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('renders action type badges', () => {
    const taskWithActions: UnitTask = {
      ...mockTask,
      status: 'action_required',
      actionTypes: ['review_requested'],
    };
    render(<UnitTaskCard task={taskWithActions} />);
    expect(screen.getByText('Review')).toBeTruthy();
    expect(screen.getByText('Action Required')).toBeTruthy();
  });

  it('renders branch name when present', () => {
    const taskWithBranch: UnitTask = {
      ...mockTask,
      branchName: 'feat/fix-login-bug',
    };
    render(<UnitTaskCard task={taskWithBranch} />);
    expect(screen.getByText('feat/fix-login-bug')).toBeTruthy();
  });

  it('renders destructive badge for failed status', () => {
    const failedTask: UnitTask = {
      ...mockTask,
      status: 'failed',
    };
    render(<UnitTaskCard task={failedTask} />);
    expect(screen.getByText('Failed')).toBeTruthy();
  });

  it('renders correct status for completed task', () => {
    const completedTask: UnitTask = {
      ...mockTask,
      status: 'completed',
    };
    render(<UnitTaskCard task={completedTask} />);
    expect(screen.getByText('Completed')).toBeTruthy();
  });
});
