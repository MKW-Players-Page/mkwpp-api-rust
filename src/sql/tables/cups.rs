// Pseudo-Table that will be actually added in a migration if necessary

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Cups {
    pub id: i32,
    pub code: &'static str,
}

impl Cups {
    pub fn reg_track_cups() -> Vec<Self> {
        return vec![
            Cups {
                id: 1,
                code: "mushroom",
            },
            Cups {
                id: 2,
                code: "flower",
            },
            Cups {
                id: 3,
                code: "star",
            },
            Cups {
                id: 4,
                code: "special",
            },
            Cups {
                id: 5,
                code: "shell",
            },
            Cups {
                id: 6,
                code: "banana",
            },
            Cups {
                id: 7,
                code: "leaf",
            },
            Cups {
                id: 8,
                code: "lightning",
            },
        ];
    }
}
