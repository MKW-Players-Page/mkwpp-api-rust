// Pseudo-Table that will be actually added in a migration if necessary

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cups {
    pub id: i32,
    pub code: &'static str,
    pub track_ids: [i32; 4],
}

impl Cups {
    pub fn reg_track_cups() -> Vec<Self> {
        vec![
            Cups {
                id: 1,
                code: "MUSHROOM",
                track_ids: [4 - 3, 4 - 2, 4 - 1, 4],
            },
            Cups {
                id: 2,
                code: "FLOWER",
                track_ids: [2 * 4 - 3, 2 * 4 - 2, 2 * 4 - 1, 2 * 4],
            },
            Cups {
                id: 3,
                code: "STAR",
                track_ids: [3 * 4 - 3, 3 * 4 - 2, 3 * 4 - 1, 3 * 4],
            },
            Cups {
                id: 4,
                code: "SPECIAL",
                track_ids: [4 * 4 - 3, 4 * 4 - 2, 4 * 4 - 1, 4 * 4],
            },
            Cups {
                id: 5,
                code: "SHELL",
                track_ids: [5 * 4 - 3, 5 * 4 - 2, 5 * 4 - 1, 5 * 4],
            },
            Cups {
                id: 6,
                code: "BANANA",
                track_ids: [6 * 4 - 3, 6 * 4 - 2, 6 * 4 - 1, 6 * 4],
            },
            Cups {
                id: 7,
                code: "LEAF",
                track_ids: [7 * 4 - 3, 7 * 4 - 2, 7 * 4 - 1, 7 * 4],
            },
            Cups {
                id: 8,
                code: "LIGHTNING",
                track_ids: [8 * 4 - 3, 8 * 4 - 2, 8 * 4 - 1, 8 * 4],
            },
        ]
    }
}
