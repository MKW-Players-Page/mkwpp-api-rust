// Pseudo-Table that will be actually added in a migration if necessary

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Cups {
    pub id: i32,
    pub code: &'static str,
    pub tracks: [i32; 4],
}

impl Cups {
    pub fn reg_track_cups() -> Vec<Self> {
        return vec![
            Cups {
                id: 1,
                code: "mushroom",
                tracks: [4 - 3, 4 - 2, 4 - 1, 4],
            },
            Cups {
                id: 2,
                code: "flower",
                tracks: [2 * 4 - 3, 2 * 4 - 2, 2 * 4 - 1, 2 * 4],
            },
            Cups {
                id: 3,
                code: "star",
                tracks: [3 * 4 - 3, 3 * 4 - 2, 3 * 4 - 1, 3 * 4],
            },
            Cups {
                id: 4,
                code: "special",
                tracks: [4 * 4 - 3, 4 * 4 - 2, 4 * 4 - 1, 4 * 4],
            },
            Cups {
                id: 5,
                code: "shell",
                tracks: [5 * 4 - 3, 5 * 4 - 2, 5 * 4 - 1, 5 * 4],
            },
            Cups {
                id: 6,
                code: "banana",
                tracks: [6 * 4 - 3, 6 * 4 - 2, 6 * 4 - 1, 6 * 4],
            },
            Cups {
                id: 7,
                code: "leaf",
                tracks: [7 * 4 - 3, 7 * 4 - 2, 7 * 4 - 1, 7 * 4],
            },
            Cups {
                id: 8,
                code: "lightning",
                tracks: [8 * 4 - 3, 8 * 4 - 2, 8 * 4 - 1, 8 * 4],
            },
        ];
    }
}
