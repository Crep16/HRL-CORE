import { computed, ref } from "vue";
import { initialOrders } from "../data/orders";
import type { Order, SearchField } from "../types/order";

export function useOrders() {
  const orders = ref<Order[]>(initialOrders);
  const selectedId = ref(initialOrders[0].id);
  const query = ref("");
  const field = ref<SearchField>("all");
  const ascending = ref(true);

  const sorted = computed(() =>
    [...orders.value].sort((a, b) => (ascending.value ? a.id.localeCompare(b.id) : b.id.localeCompare(a.id))),
  );

  const visible = computed(() => {
    const text = query.value.trim().toLocaleLowerCase("es");
    if (!text) return sorted.value;
    return sorted.value.filter((order) => {
      const values = {
        all: [order.id, order.client, order.serial],
        order: [order.id],
        client: [order.client],
        serial: [order.serial],
      }[field.value];
      return values.some((value) => value.toLocaleLowerCase("es").includes(text));
    });
  });

  const index = computed(() => sorted.value.findIndex((order) => order.id === selectedId.value));
  const selected = computed<Order | undefined>(() => sorted.value[index.value]);

  function goTo(position: number) {
    const list = sorted.value;
    if (!list.length) return;
    selectedId.value = list[Math.min(Math.max(position, 0), list.length - 1)].id;
  }

  function removeSelected() {
    const position = index.value;
    orders.value = orders.value.filter((order) => order.id !== selectedId.value);
    goTo(position);
  }

  return { orders, selectedId, query, field, ascending, visible, index, selected, goTo, removeSelected };
}
