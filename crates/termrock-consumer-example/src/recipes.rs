//! The consumer's own domain: a recipe shelf.
//!
//! Nothing here is shared with any application in this repository. The data
//! is deliberately unrelated to the showcase, Holla, TablePro or Jackin
//! domains so the consumer proves that the Termrock components carry an
//! arbitrary payload, not one application's vocabulary.

use termrock::ItemKey;

/// One labelled fact about a recipe, rendered as a property row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fact {
    /// The stable key of this fact inside its recipe.
    pub key: u64,
    /// The left-hand label of the property row.
    pub label: &'static str,
    /// The right-hand value of the property row.
    pub value: &'static str,
}

/// One recipe on the shelf.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipe {
    /// The stable identity of the recipe.
    pub id: u64,
    /// The name shown in the recipe list.
    pub name: &'static str,
    /// The section the recipe is filed under.
    pub course: &'static str,
    /// The number of facts the recipe carries.
    pub facts: &'static [Fact],
}

impl Recipe {
    /// The stable key the shared list widgets address this recipe by.
    pub const fn key(&self) -> ItemKey {
        ItemKey::num(self.id)
    }
}

/// The shelf this consumer ships with.
pub const RECIPES: &[Recipe] = &[
    Recipe {
        id: 101,
        name: "Miso Butter Ramen",
        course: "Soup",
        facts: &[
            Fact {
                key: 1,
                label: "Broth",
                value: "chicken · kombu · white miso",
            },
            Fact {
                key: 2,
                label: "Noodles",
                value: "fresh wheat, 90 g",
            },
            Fact {
                key: 3,
                label: "Finish",
                value: "butter · scallion · nori",
            },
        ],
    },
    Recipe {
        id: 102,
        name: "Roasted Squash Soup",
        course: "Soup",
        facts: &[
            Fact {
                key: 1,
                label: "Roast",
                value: "200 °C, 35 min",
            },
            Fact {
                key: 2,
                label: "Base",
                value: "onion · garlic · thyme",
            },
            Fact {
                key: 3,
                label: "Finish",
                value: "brown butter · pumpkin seeds",
            },
        ],
    },
    Recipe {
        id: 103,
        name: "Chickpea Stew",
        course: "Mains",
        facts: &[
            Fact {
                key: 1,
                label: "Base",
                value: "tomato · onion · cumin",
            },
            Fact {
                key: 2,
                label: "Protein",
                value: "chickpeas · preserved lemon",
            },
            Fact {
                key: 3,
                label: "Finish",
                value: "coriander · olive oil",
            },
        ],
    },
    Recipe {
        id: 104,
        name: "Mushroom Risotto",
        course: "Mains",
        facts: &[
            Fact {
                key: 1,
                label: "Rice",
                value: "carnaroli, 320 g",
            },
            Fact {
                key: 2,
                label: "Stock",
                value: "mushroom · thyme, warm",
            },
            Fact {
                key: 3,
                label: "Finish",
                value: "parmesan · chives",
            },
        ],
    },
    Recipe {
        id: 105,
        name: "Olive Oil Cake",
        course: "Baking",
        facts: &[
            Fact {
                key: 1,
                label: "Dough",
                value: "flour · sugar · orange zest",
            },
            Fact {
                key: 2,
                label: "Fat",
                value: "olive oil, 120 ml",
            },
            Fact {
                key: 3,
                label: "Bake",
                value: "170 °C, 45 min",
            },
        ],
    },
];
