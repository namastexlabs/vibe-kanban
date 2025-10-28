// useConversationHistory.ts
import {
  CommandExitStatus,
  ExecutionProcess,
  ExecutionProcessStatus,
  ExecutorAction,
  NormalizedEntry,
  PatchType,
  TaskAttempt,
  ToolStatus,
} from 'shared/types';
import { useExecutionProcessesContext } from '@/contexts/ExecutionProcessesContext';
import { useEffect, useMemo, useRef } from 'react';
import { streamJsonPatchEntries } from '@/utils/streamJsonPatchEntries';

export type PatchTypeWithKey = PatchType & {
  patchKey: string;
  executionProcessId: string;
};

export type AddEntryType = 'initial' | 'running' | 'historic';

export type OnEntriesUpdated = (
  newEntries: PatchTypeWithKey[],
  addType: AddEntryType,
  loading: boolean
) => void;

type ExecutionProcessStaticInfo = {
  id: string;
  created_at: string;
  updated_at: string;
  executor_action: ExecutorAction;
};

type ExecutionProcessState = {
  executionProcess: ExecutionProcessStaticInfo;
  entries: PatchTypeWithKey[];
};

type ExecutionProcessStateStore = Record<string, ExecutionProcessState>;

interface UseConversationHistoryParams {
  attempt: TaskAttempt;
  onEntriesUpdated: OnEntriesUpdated;
}

interface UseConversationHistoryResult {}

const MIN_INITIAL_ENTRIES = 10;
const REMAINING_BATCH_SIZE = 50;

export const useConversationHistory = ({
  attempt,
  onEntriesUpdated,
}: UseConversationHistoryParams): UseConversationHistoryResult => {
  const { executionProcessesVisible: executionProcessesRaw } =
    useExecutionProcessesContext();
  const executionProcesses = useRef<ExecutionProcess[]>(executionProcessesRaw);
  const displayedExecutionProcesses = useRef<ExecutionProcessStateStore>({});
  const loadedInitialEntries = useRef(false);
  const lastActiveProcessId = useRef<string | null>(null);
  const onEntriesUpdatedRef = useRef<OnEntriesUpdated | null>(null);

  const mergeIntoDisplayed = (
    mutator: (state: ExecutionProcessStateStore) => void
  ) => {
    const state = displayedExecutionProcesses.current;
    mutator(state);
  };
  useEffect(() => {
    onEntriesUpdatedRef.current = onEntriesUpdated;
  }, [onEntriesUpdated]);

  // Keep executionProcesses up to date
  useEffect(() => {
    executionProcesses.current = executionProcessesRaw;
  }, [executionProcessesRaw]);

  const loadEntriesForHistoricExecutionProcess = (
    executionProcess: ExecutionProcess
  ) => {
    let url = '';
    if (executionProcess.executor_action.typ.type === 'ScriptRequest') {
      url = `/api/execution-processes/${executionProcess.id}/raw-logs/ws`;
    } else {
      url = `/api/execution-processes/${executionProcess.id}/normalized-logs/ws`;
    }

    return new Promise<PatchType[]>((resolve) => {
      const controller = streamJsonPatchEntries<PatchType>(url, {
        onFinished: (allEntries) => {
          controller.close();
          resolve(allEntries);
        },
        onError: (err) => {
          // WebSocket failures for historic processes are expected and harmless
          // Use console.debug to avoid console spam during normal operation
          console.debug(
            `WebSocket connection failed for historic execution process ${executionProcess.id} (expected for completed processes)`,
            err
          );
          controller.close();
          resolve([]);
        },
      });
    });
  };

  const getLiveExecutionProcess = (
    executionProcessId: string
  ): ExecutionProcess | undefined => {
    return executionProcesses?.current.find(
      (executionProcess) => executionProcess.id === executionProcessId
    );
  };

  // This emits its own events as they are streamed
  const loadRunningAndEmit = (
    executionProcess: ExecutionProcess
  ): Promise<void> => {
    return new Promise((resolve, reject) => {
      let url = '';
      if (executionProcess.executor_action.typ.type === 'ScriptRequest') {
        url = `/api/execution-processes/${executionProcess.id}/raw-logs/ws`;
      } else {
        url = `/api/execution-processes/${executionProcess.id}/normalized-logs/ws`;
      }
      const controller = streamJsonPatchEntries<PatchType>(url, {
        onEntries(entries) {
          const patchesWithKey = entries.map((entry, index) =>
            patchWithKey(entry, executionProcess.id, index)
          );
          mergeIntoDisplayed((state) => {
            state[executionProcess.id] = {
              executionProcess,
              entries: patchesWithKey,
            };
          });
          emitEntries(displayedExecutionProcesses.current, 'running', false);
        },
        onFinished: () => {
          emitEntries(displayedExecutionProcesses.current, 'running', false);
          controller.close();
          resolve();
        },
        onError: () => {
          controller.close();
          reject();
        },
      });
    });
  };

  // Sometimes it can take a few seconds for the stream to start, wrap the loadRunningAndEmit method
  const loadRunningAndEmitWithBackoff = async (
    executionProcess: ExecutionProcess
  ) => {
    for (let i = 0; i < 20; i++) {
      try {
        await loadRunningAndEmit(executionProcess);
        break;
      } catch (_) {
        await new Promise((resolve) => setTimeout(resolve, 500));
      }
    }
  };

  const getActiveAgentProcess = (): ExecutionProcess | null => {
    const activeProcesses = executionProcesses?.current.filter(
      (p) =>
        p.status === ExecutionProcessStatus.running &&
        p.run_reason !== 'devserver'
    );
    if (activeProcesses.length > 1) {
      const processIds = activeProcesses.map(p => p.id).join(', ');
      console.error(`More than one active execution process found. Process IDs: ${processIds}`);
    }
    return activeProcesses[0] || null;
  };

  const flattenEntries = (
    executionProcessState: ExecutionProcessStateStore
  ): PatchTypeWithKey[] => {
    return Object.values(executionProcessState)
      .filter(
        (p) =>
          p.executionProcess.executor_action.typ.type ===
            'CodingAgentFollowUpRequest' ||
          p.executionProcess.executor_action.typ.type ===
            'CodingAgentInitialRequest'
      )
      .sort(
        (a, b) =>
          new Date(
            a.executionProcess.created_at as unknown as string
          ).getTime() -
          new Date(b.executionProcess.created_at as unknown as string).getTime()
      )
      .flatMap((p) => p.entries);
  };

  const loadingPatch: PatchTypeWithKey = {
    type: 'NORMALIZED_ENTRY',
    content: {
      entry_type: {
        type: 'loading',
      },
      content: '',
      timestamp: null,
    },
    patchKey: 'loading',
    executionProcessId: '',
  };

  const flattenEntriesForEmit = (
    executionProcessState: ExecutionProcessStateStore
  ): PatchTypeWithKey[] => {
    // Create user messages + tool calls for setup/cleanup scripts
    const allEntries = Object.values(executionProcessState)
      .sort(
        (a, b) =>
          new Date(
            a.executionProcess.created_at as unknown as string
          ).getTime() -
          new Date(b.executionProcess.created_at as unknown as string).getTime()
      )
      .flatMap((p) => {
        const entries: PatchTypeWithKey[] = [];
        if (
          p.executionProcess.executor_action.typ.type ===
            'CodingAgentInitialRequest' ||
          p.executionProcess.executor_action.typ.type ===
            'CodingAgentFollowUpRequest'
        ) {
          // New user message
          const userNormalizedEntry: NormalizedEntry = {
            entry_type: {
              type: 'user_message',
            },
            content: p.executionProcess.executor_action.typ.prompt,
            timestamp: null,
          };
          const userPatch: PatchType = {
            type: 'NORMALIZED_ENTRY',
            content: userNormalizedEntry,
          };
          const userPatchTypeWithKey = patchWithKey(
            userPatch,
            p.executionProcess.id,
            'user'
          );
          entries.push(userPatchTypeWithKey);

          // Remove all coding agent added user messages, replace with our custom one
          const entriesExcludingUser = p.entries.filter(
            (e) =>
              e.type !== 'NORMALIZED_ENTRY' ||
              e.content.entry_type.type !== 'user_message'
          );

          const hasPendingApprovalEntry = entriesExcludingUser.some((entry) => {
            if (entry.type !== 'NORMALIZED_ENTRY') return false;
            const entryType = entry.content.entry_type;
            return (
              entryType.type === 'tool_use' &&
              entryType.status.status === 'pending_approval'
            );
          });

          entries.push(...entriesExcludingUser);
          const isProcessRunning =
            getLiveExecutionProcess(p.executionProcess.id)?.status ===
            ExecutionProcessStatus.running;

          if (isProcessRunning && !hasPendingApprovalEntry) {
            entries.push(loadingPatch);
          }
        } else if (
          p.executionProcess.executor_action.typ.type === 'ScriptRequest'
        ) {
          // Add setup and cleanup script as a tool call
          let toolName = '';
          switch (p.executionProcess.executor_action.typ.context) {
            case 'SetupScript':
              toolName = 'Setup Script';
              break;
            case 'CleanupScript':
              toolName = 'Cleanup Script';
              break;
            default:
              return [];
          }

          const executionProcess = getLiveExecutionProcess(
            p.executionProcess.id
          );

          const exitCode = Number(executionProcess?.exit_code) || 0;
          const exit_status: CommandExitStatus | null =
            executionProcess?.status === 'running'
              ? null
              : {
                  type: 'exit_code',
                  code: exitCode,
                };

          const toolStatus: ToolStatus =
            executionProcess?.status === ExecutionProcessStatus.running
              ? { status: 'created' }
              : exitCode === 0
                ? { status: 'success' }
                : { status: 'failed' };

          const output = p.entries.map((line) => line.content).join('\n');

          const toolNormalizedEntry: NormalizedEntry = {
            entry_type: {
              type: 'tool_use',
              tool_name: toolName,
              action_type: {
                action: 'command_run',
                command: p.executionProcess.executor_action.typ.script,
                result: {
                  output,
                  exit_status,
                },
              },
              status: toolStatus,
            },
            content: toolName,
            timestamp: null,
          };
          const toolPatch: PatchType = {
            type: 'NORMALIZED_ENTRY',
            content: toolNormalizedEntry,
          };
          const toolPatchWithKey: PatchTypeWithKey = patchWithKey(
            toolPatch,
            p.executionProcess.id,
            0
          );

          entries.push(toolPatchWithKey);
        }

        return entries;
      });

    return allEntries;
  };

  const patchWithKey = (
    patch: PatchType,
    executionProcessId: string,
    index: number | 'user'
  ) => {
    return {
      ...patch,
      patchKey: `${executionProcessId}:${index}`,
      executionProcessId,
    };
  };

  const loadInitialEntries = async (): Promise<ExecutionProcessStateStore> => {
    const localDisplayedExecutionProcesses: ExecutionProcessStateStore = {};

    if (!executionProcesses?.current) return localDisplayedExecutionProcesses;

    for (const executionProcess of [...executionProcesses.current].reverse()) {
      if (executionProcess.status === ExecutionProcessStatus.running) continue;

      const entries =
        await loadEntriesForHistoricExecutionProcess(executionProcess);
      const entriesWithKey = entries.map((e, idx) =>
        patchWithKey(e, executionProcess.id, idx)
      );

      localDisplayedExecutionProcesses[executionProcess.id] = {
        executionProcess,
        entries: entriesWithKey,
      };

      if (
        flattenEntries(localDisplayedExecutionProcesses).length >
        MIN_INITIAL_ENTRIES
      ) {
        break;
      }
    }

    return localDisplayedExecutionProcesses;
  };

  const loadRemainingEntriesInBatches = async (
    batchSize: number
  ): Promise<boolean> => {
    if (!executionProcesses?.current) return false;

    let anyUpdated = false;
    for (const executionProcess of [...executionProcesses.current].reverse()) {
      const current = displayedExecutionProcesses.current;
      if (
        current[executionProcess.id] ||
        executionProcess.status === ExecutionProcessStatus.running
      )
        continue;

      const entries =
        await loadEntriesForHistoricExecutionProcess(executionProcess);
      const entriesWithKey = entries.map((e, idx) =>
        patchWithKey(e, executionProcess.id, idx)
      );

      mergeIntoDisplayed((state) => {
        state[executionProcess.id] = {
          executionProcess,
          entries: entriesWithKey,
        };
      });

      if (
        flattenEntries(displayedExecutionProcesses.current).length > batchSize
      ) {
        anyUpdated = true;
        break;
      }
      anyUpdated = true;
    }
    return anyUpdated;
  };

  const emitEntries = (
    executionProcessState: ExecutionProcessStateStore,
    addEntryType: AddEntryType,
    loading: boolean
  ) => {
    const entries = flattenEntriesForEmit(executionProcessState);
    onEntriesUpdatedRef.current?.(entries, addEntryType, loading);
  };

  const ensureProcessVisible = (p: ExecutionProcess) => {
    mergeIntoDisplayed((state) => {
      if (!state[p.id]) {
        state[p.id] = {
          executionProcess: {
            id: p.id,
            created_at: p.created_at,
            updated_at: p.updated_at,
            executor_action: p.executor_action,
          },
          entries: [],
        };
      }
    });
  };

  const idListKey = useMemo(
    () => executionProcessesRaw?.map((p) => p.id).join(','),
    [executionProcessesRaw]
  );

  const idStatusKey = useMemo(
    () => executionProcessesRaw?.map((p) => `${p.id}:${p.status}`).join(','),
    [executionProcessesRaw]
  );

  // Initial load when attempt changes
  useEffect(() => {
    let cancelled = false;
    (async () => {
      // Waiting for execution processes to load
      if (
        executionProcesses?.current.length === 0 ||
        loadedInitialEntries.current
      )
        return;

      // Initial entries
      const allInitialEntries = await loadInitialEntries();
      if (cancelled) return;
      mergeIntoDisplayed((state) => {
        Object.assign(state, allInitialEntries);
      });
      emitEntries(displayedExecutionProcesses.current, 'initial', false);
      loadedInitialEntries.current = true;

      // Then load the remaining in batches
      while (
        !cancelled &&
        (await loadRemainingEntriesInBatches(REMAINING_BATCH_SIZE))
      ) {
        if (cancelled) return;
      }
      await new Promise((resolve) => setTimeout(resolve, 100));
      emitEntries(displayedExecutionProcesses.current, 'historic', false);
    })();
    return () => {
      cancelled = true;
    };
  }, [attempt.id, idListKey]); // include idListKey so new processes trigger reload

  useEffect(() => {
    const activeProcess = getActiveAgentProcess();
    if (!activeProcess) return;

    if (!displayedExecutionProcesses.current[activeProcess.id]) {
      ensureProcessVisible(activeProcess);
      emitEntries(displayedExecutionProcesses.current, 'running', false);
    }

    if (
      activeProcess.status === ExecutionProcessStatus.running &&
      lastActiveProcessId.current !== activeProcess.id
    ) {
      lastActiveProcessId.current = activeProcess.id;
      loadRunningAndEmitWithBackoff(activeProcess);
    }
  }, [attempt.id, idStatusKey]);

  // If an execution process is removed, remove it from the state
  useEffect(() => {
    if (!executionProcessesRaw) return;

    const removedProcessIds = Object.keys(
      displayedExecutionProcesses.current
    ).filter((id) => !executionProcessesRaw.some((p) => p.id === id));

    if (removedProcessIds.length > 0) {
      mergeIntoDisplayed((state) => {
        removedProcessIds.forEach((id) => {
          delete state[id];
        });
      });
    }
  }, [attempt.id, idListKey]);

  // Reset state when attempt changes
  useEffect(() => {
    displayedExecutionProcesses.current = {};
    loadedInitialEntries.current = false;
    lastActiveProcessId.current = null;
    emitEntries(displayedExecutionProcesses.current, 'initial', true);
  }, [attempt.id]);

  return {};
};
