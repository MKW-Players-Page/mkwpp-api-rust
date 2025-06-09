use crate::sql::tables::{
    BasicTableQueries,
    regions::{Regions, RegionsWithPlayerCount},
};
use actix_web::HttpResponse;
use serde::ser::SerializeMap;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ChildrenTree {
    pub id: i32,
    pub children: Option<Vec<ChildrenTree>>,

    dangling: Option<HashMap<i32, Vec<ChildrenTree>>>,
    is_root: bool,
}

impl ChildrenTree {
    fn new(id: i32) -> Self {
        Self {
            id,
            children: None,
            dangling: None,
            is_root: false,
        }
    }

    fn insert(&mut self, key: i32, mut value: ChildrenTree) -> Result<(), ChildrenTree> {
        if self.is_root {
            match self.dangling {
                None => self.dangling = Some(HashMap::new()),
                Some(ref mut dangling_hashmap) => {
                    if let Some(dangling) = dangling_hashmap.remove(&value.id) {
                        match value.children {
                            None => value.children = Some(dangling),
                            Some(ref mut x) => x.extend(dangling),
                        }
                    }
                }
            }
        }

        if self.id == key {
            match self.children {
                None => self.children = Some(vec![value]),
                Some(ref mut z) => z.push(value),
            }
            return Ok(());
        }

        if let Some(ref mut children) = self.children {
            for child in children {
                match child.insert(key, value) {
                    Ok(_) => return Ok(()),
                    Err(v) => value = v,
                }
            }
        }

        if self.is_root {
            if let Some(ref mut dangling_hashmap) = self.dangling {
                match dangling_hashmap.get_mut(&key) {
                    Some(v) => v.push(value),
                    None => {
                        for vector in dangling_hashmap.values_mut() {
                            for child in vector {
                                match child.insert(key, value) {
                                    Ok(_) => return Ok(()),
                                    Err(v) => value = v,
                                }
                            }
                        }
                        dangling_hashmap.insert(key, vec![value]);
                    }
                }
            }
            return Ok(());
        }

        Err(value)
    }
}

impl serde::Serialize for ChildrenTree {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.children {
            None => serializer.serialize_i32(self.id),
            Some(ref v) => {
                let mut z = serializer.serialize_map(Some(1))?;
                z.serialize_entry(&self.id, v)?;
                z.end()
            }
        }
    }
}

macro_rules! region_tree_generator {
    ($region_subtype:ident, $fn_name:ident) => {
        pub async fn $fn_name(data: Vec<$region_subtype>) -> ChildrenTree {
            let mut tree: ChildrenTree = ChildrenTree {
                id: 1,
                children: Some(vec![]),
                dangling: Some(HashMap::new()),
                is_root: true,
            };

            for region in data {
                if let Some(parent_id) = region.parent_id {
                    _ = tree.insert(parent_id, ChildrenTree::new(region.id));
                }
            }

            tree
        }
    };
}

region_tree_generator!(Regions, generate_region_tree);
region_tree_generator!(RegionsWithPlayerCount, generate_region_tree_player_count);

pub async fn get_region_child_tree() -> HttpResponse {
    crate::api::v1::basic_get_with_data_mod::<Regions, ChildrenTree>(
        Regions::select_star_query,
        generate_region_tree,
    )
    .await
}
