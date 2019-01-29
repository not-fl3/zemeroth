use crate::core::{
    map::{self, PosHex},
    tactical_map::{ability::PassiveAbility, utils, ObjId, PlayerId, Strength, TileType},
};

pub use self::private::{BattleResult, State};

mod private {
    use log::error;

    use crate::core::{
        map::{self, HexMap},
        tactical_map::{
            command::{self, Command},
            component::{Component, Parts, Prototypes},
            execute,
            scenario::{self, Scenario},
            ObjId, PlayerId, TileType,
        },
    };

    #[derive(Clone, Debug)]
    pub struct BattleResult {
        pub winner_id: PlayerId,
        pub survivor_types: Vec<String>,
    }

    #[derive(Clone, Debug)]
    pub struct State {
        parts: Parts,
        map: map::HexMap<TileType>,
        scenario: Scenario,
        player_id: PlayerId,
        prototypes: Prototypes,
        battle_result: Option<BattleResult>,
    }

    impl State {
        pub fn new(prototypes: Prototypes, scenario: Scenario, cb: execute::Cb) -> Self {
            assert_eq!(scenario.players_count, 2, "Only 2 players are supported");
            assert!(scenario.map_radius.0 >= 3);
            let mut this = Self {
                map: map::HexMap::new(scenario.map_radius),
                player_id: PlayerId(0),
                scenario,
                parts: Parts::new(),
                prototypes,
                battle_result: None,
            };
            this.create_terrain();
            this.create_objects(cb);
            this
        }

        pub fn scenario(&self) -> &Scenario {
            &self.scenario
        }

        fn create_terrain(&mut self) {
            for _ in 0..self.scenario.rocky_tiles_count {
                let pos = match scenario::random_free_pos(self) {
                    Some(pos) => pos,
                    None => continue,
                };
                self.map_mut().set_tile(pos, TileType::Rocks);
            }
        }

        fn create_objects(&mut self, cb: execute::Cb) {
            let player_id_initial = self.player_id();
            for group in self.scenario.objects.clone() {
                if let Some(player_id) = group.owner {
                    self.set_player_id(player_id);
                }
                for _ in 0..group.count {
                    let pos = match scenario::random_pos(self, group.owner, group.line) {
                        Some(pos) => pos,
                        None => {
                            error!("Can't find the position");
                            continue;
                        }
                    };
                    let command = Command::Create(command::Create {
                        prototype: group.typename.clone(),
                        pos,
                        owner: group.owner,
                    });
                    execute::execute(self, &command, cb).expect("Can't create an object");
                }
            }
            self.set_player_id(player_id_initial);
        }

        pub fn player_id(&self) -> PlayerId {
            self.player_id
        }

        pub fn next_player_id(&self) -> PlayerId {
            let current_player_id = PlayerId(self.player_id().0 + 1);
            if current_player_id.0 < self.scenario.players_count {
                current_player_id
            } else {
                PlayerId(0)
            }
        }

        pub fn parts(&self) -> &Parts {
            &self.parts
        }

        pub fn map(&self) -> &map::HexMap<TileType> {
            &self.map
        }

        // TODO: make visible only for `apply`
        pub(in crate::core) fn prototype_for(&self, name: &str) -> Vec<Component> {
            let prototypes = &self.prototypes.0;
            prototypes[name].clone()
        }

        pub fn battle_result(&self) -> &Option<BattleResult> {
            &self.battle_result
        }
    }

    /// Mutators. Be carefull with them!
    impl State {
        // TODO: check that it's called only from apply.rs!
        pub(in crate::core) fn parts_mut(&mut self) -> &mut Parts {
            &mut self.parts
        }

        pub(in crate::core) fn map_mut(&mut self) -> &mut HexMap<TileType> {
            &mut self.map
        }

        pub(in crate::core) fn set_player_id(&mut self, new_value: PlayerId) {
            self.player_id = new_value;
        }

        pub(in crate::core) fn set_battle_result(&mut self, result: BattleResult) {
            self.battle_result = Some(result);
        }

        pub(in crate::core) fn alloc_id(&mut self) -> ObjId {
            self.parts.alloc_id()
        }
    }
}

pub fn is_agent_belong_to(state: &State, player_id: PlayerId, id: ObjId) -> bool {
    state.parts().belongs_to.get(id).0 == player_id
}

pub fn is_tile_blocked(state: &State, pos: PosHex) -> bool {
    assert!(state.map().is_inboard(pos));
    for id in state.parts().blocker.ids() {
        if state.parts().pos.get(id).0 == pos {
            return true;
        }
    }
    false
}

pub fn is_tile_plain_and_completely_free(state: &State, pos: PosHex) -> bool {
    if !state.map().is_inboard(pos) || state.map().tile(pos) != TileType::Plain {
        return false;
    }
    for id in state.parts().pos.ids() {
        if state.parts().pos.get(id).0 == pos {
            return false;
        }
    }
    true
}

pub fn is_tile_completely_free(state: &State, pos: PosHex) -> bool {
    if !state.map().is_inboard(pos) {
        return false;
    }
    for id in state.parts().pos.ids() {
        if state.parts().pos.get(id).0 == pos {
            return false;
        }
    }
    true
}

/// Are there any enemy agents on the adjacent tiles?
pub fn check_enemies_around(state: &State, pos: PosHex, player_id: PlayerId) -> bool {
    for dir in map::dirs() {
        let neighbor_pos = map::Dir::get_neighbor_pos(pos, dir);
        if let Some(id) = agent_id_at_opt(state, neighbor_pos) {
            let neighbor_player_id = state.parts().belongs_to.get(id).0;
            if neighbor_player_id != player_id {
                return true;
            }
        }
    }
    false
}

pub fn ids_at(state: &State, pos: PosHex) -> Vec<ObjId> {
    let i = state.parts().pos.ids();
    i.filter(|&id| state.parts().pos.get(id).0 == pos).collect()
}

pub fn obj_with_passive_ability_at(
    state: &State,
    pos: PosHex,
    ability: PassiveAbility,
) -> Option<ObjId> {
    for id in ids_at(state, pos) {
        if let Some(abilities) = state.parts().passive_abilities.get_opt(id) {
            for &current_ability in &abilities.0 {
                if current_ability == ability {
                    return Some(id);
                }
            }
        }
    }
    None
}

pub fn blocker_id_at(state: &State, pos: PosHex) -> ObjId {
    blocker_id_at_opt(state, pos).unwrap()
}

pub fn blocker_id_at_opt(state: &State, pos: PosHex) -> Option<ObjId> {
    let ids = blocker_ids_at(state, pos);
    if ids.len() == 1 {
        Some(ids[0])
    } else {
        None
    }
}

pub fn agent_id_at_opt(state: &State, pos: PosHex) -> Option<ObjId> {
    let ids = agent_ids_at(state, pos);
    if ids.len() == 1 {
        Some(ids[0])
    } else {
        None
    }
}

pub fn agent_ids_at(state: &State, pos: PosHex) -> Vec<ObjId> {
    let i = state.parts().agent.ids();
    i.filter(|&id| state.parts().pos.get(id).0 == pos).collect()
}

pub fn blocker_ids_at(state: &State, pos: PosHex) -> Vec<ObjId> {
    let i = state.parts().blocker.ids();
    i.filter(|&id| state.parts().pos.get(id).0 == pos).collect()
}

pub fn players_agent_ids(state: &State, player_id: PlayerId) -> Vec<ObjId> {
    let i = state.parts().agent.ids();
    i.filter(|&id| is_agent_belong_to(state, player_id, id))
        .collect()
}

pub fn enemy_agent_ids(state: &State, player_id: PlayerId) -> Vec<ObjId> {
    let i = state.parts().agent.ids();
    i.filter(|&id| !is_agent_belong_to(state, player_id, id))
        .collect()
}

pub fn free_neighbor_positions(state: &State, origin: PosHex, count: i32) -> Vec<PosHex> {
    let mut positions = Vec::new();
    for dir in utils::shuffle_vec(map::dirs().collect()) {
        let pos = map::Dir::get_neighbor_pos(origin, dir);
        if state.map().is_inboard(pos) && !is_tile_blocked(state, pos) {
            positions.push(pos);
            if positions.len() == count as usize {
                break;
            }
        }
    }
    positions
}

pub fn sort_agent_ids_by_distance_to_enemies(state: &State, ids: &mut [ObjId]) {
    ids.sort_unstable_by_key(|&id| {
        let agent_player_id = state.parts().belongs_to.get(id).0;
        let agent_pos = state.parts().pos.get(id).0;
        let mut min_distance = state.map().height();
        for enemy_id in enemy_agent_ids(state, agent_player_id) {
            let enemy_pos = state.parts().pos.get(enemy_id).0;
            let distance = map::distance_hex(agent_pos, enemy_pos);
            if distance < min_distance {
                min_distance = distance;
            }
        }
        min_distance
    });
}

pub fn get_armor(state: &State, id: ObjId) -> Strength {
    let parts = state.parts();
    let default = Strength(0);
    parts.armor.get_opt(id).map(|v| v.armor).unwrap_or(default)
}

pub fn players_agent_types(state: &State, player_id: PlayerId) -> Vec<String> {
    players_agent_ids(state, player_id)
        .into_iter()
        .map(|id| state.parts().meta.get(id).name.clone())
        .collect()
}
