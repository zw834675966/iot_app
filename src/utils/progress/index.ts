type ProgressApi = {
  start: () => void;
  done: () => void;
  configure: (_options?: Record<string, unknown>) => ProgressApi;
};

const progress: ProgressApi = {
  start: () => {
    // no-op: keep compatibility after removing nprogress dependency
  },
  done: () => {
    // no-op: keep compatibility after removing nprogress dependency
  },
  configure: () => progress
};

export default progress;
