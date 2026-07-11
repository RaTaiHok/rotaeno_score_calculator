import { onMounted, ref, watch } from "vue";

const STORAGE_KEY = "rotaeno_calc_history";
const MAX_ENTRIES = 100;

const history = ref([]);

function generateId() {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 8);
}

function load() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      history.value = JSON.parse(raw);
    }
  } catch {
    history.value = [];
  }
}

function save() {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(history.value));
  } catch {
    // localStorage full or unavailable — trim old entries
    history.value = history.value.slice(0, Math.floor(MAX_ENTRIES / 2));
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(history.value));
    } catch {
      // give up silently
    }
  }
}

export function useHistory() {
  onMounted(load);

  watch(history, save, { deep: true });

  function addEntry(entry) {
    const item = {
      id: generateId(),
      timestamp: Date.now(),
      ...entry
    };

    history.value.unshift(item);

    // cap size
    if (history.value.length > MAX_ENTRIES) {
      history.value = history.value.slice(0, MAX_ENTRIES);
    }

    return item;
  }

  function removeEntry(id) {
    history.value = history.value.filter((e) => e.id !== id);
  }

  function clearHistory() {
    history.value = [];
  }

  return {
    history,
    addEntry,
    removeEntry,
    clearHistory
  };
}
