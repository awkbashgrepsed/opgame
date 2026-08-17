use crate::player::Player;
use crate::npc::NPCManager;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MissionStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mission {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: MissionStatus,
    pub reward: u32,
    pub objectives: Vec<String>,
    pub current_objective: usize,
}

impl Mission {
    pub fn new(title: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: MissionStatus::NotStarted,
            reward: 0,
            objectives: Vec::new(),
            current_objective: 0,
        }
    }

    pub fn start(&mut self) {
        self.status = MissionStatus::InProgress;
    }

    pub fn complete(&mut self) {
        self.status = MissionStatus::Completed;
    }

    pub fn fail(&mut self) {
        self.status = MissionStatus::Failed;
    }

    pub fn next_objective(&mut self) {
        self.current_objective += 1;
    }
}

pub struct MissionManager {
    missions: Vec<Mission>,
    current_mission_index: Option<usize>,
}

impl MissionManager {
    pub fn new() -> Self {
        Self {
            missions: Vec::new(),
            current_mission_index: None,
        }
    }

    pub fn create_tutorial_mission(&mut self) {
        let mut mission = Mission::new(
            "Welcome to the City".to_string(),
            "Get familiar with the city and complete some tasks.".to_string(),
        );
        mission.objectives = vec![
            "Walk around and explore".to_string(),
            "Find a weapon".to_string(),
            "Complete your first objective".to_string(),
        ];
        mission.reward = 100;
        mission.start();
        self.missions.push(mission);
        self.current_mission_index = Some(0);
    }

    pub fn add_mission(&mut self, mission: Mission) -> Uuid {
        let id = mission.id;
        self.missions.push(mission);
        id
    }

    pub fn get_current_mission(&self) -> Option<&Mission> {
        self.current_mission_index.and_then(|idx| self.missions.get(idx))
    }

    pub fn get_current_mission_mut(&mut self) -> Option<&mut Mission> {
        self.current_mission_index.and_then(|idx| self.missions.get_mut(idx))
    }

    pub fn complete_current_mission(&mut self, player: &mut Player) {
        if let Some(mission) = self.get_current_mission_mut() {
            mission.complete();
            player.add_money(mission.reward);
            player.add_experience(50);
        }
    }

    pub fn update(&mut self, player: &Player, _npc_manager: &NPCManager, _time: f32) {
        // Update mission logic
        if let Some(mission) = self.get_current_mission_mut() {
            if mission.status == MissionStatus::InProgress {
                // Check mission conditions
            }
        }
    }

    pub fn get_missions(&self) -> &[Mission] {
        &self.missions
    }
}
