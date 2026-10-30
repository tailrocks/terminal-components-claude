use termrock::{AsItem, FilterPolicy, Id, Item, ItemKey, Picker, PickerState};

struct TabSummary {
    key: u64,
    title: &'static str,
}

impl AsItem for TabSummary {
    fn as_item(&self) -> Item<'_> {
        Item::new(ItemKey::num(self.key), self.title)
    }
}

#[test]
fn external_domain_items_keep_stable_keys_through_picker_filtering() {
    const PICKER: Id = Id::root("picker.external-domain-contract");
    let items = [
        TabSummary {
            key: 41,
            title: "Orders Console",
        },
        TabSummary {
            key: 73,
            title: "Customer Notes",
        },
    ];
    let picker = Picker::new(PICKER)
        .title("Open tabs")
        .placeholder("Filter tabs…")
        .filter(FilterPolicy::Substring);
    let mut state = PickerState::default();

    picker.reconcile(&mut state, &items);
    state.set_query("console");
    picker.reconcile(&mut state, &items);

    assert_eq!(state.list().matched_len(), 1);
    assert_eq!(state.cursor(), Some(ItemKey::num(41)));
}
