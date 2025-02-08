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
                id: 0,
                code: "mushroom",
            },
            Cups {
                id: 1,
                code: "flower",
            },
            Cups {
                id: 2,
                code: "star",
            },
            Cups {
                id: 3,
                code: "special",
            },
            Cups {
                id: 4,
                code: "shell",
            },
            Cups {
                id: 5,
                code: "banana",
            },
            Cups {
                id: 6,
                code: "leaf",
            },
            Cups {
                id: 7,
                code: "lightning",
            },
        ];
    }
}
