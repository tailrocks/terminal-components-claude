use termrock::ItemKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fact {
    pub key: u64,

    pub label: &'static str,

    pub value: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub id: u64,

    pub name: &'static str,

    pub course: &'static str,

    pub facts: &'static [Fact],
}

impl Recipe {
    pub const fn key(&self) -> ItemKey {
        ItemKey::num(self.id)
    }
}

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
