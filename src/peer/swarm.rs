use std::{collections::{HashSet, VecDeque}, net::SocketAddr};

use tokio::sync::mpsc;

const TARGET_PEERS: usize = 50;

pub enum SwarmEvent {
    NewCandidates(Vec<SocketAddr>),
    PeerDisconnected(SocketAddr),
    Shutdown,
}

pub struct Swarm {
    candidates: VecDeque<SocketAddr>,
    actives: HashSet<SocketAddr>,
    checked: HashSet<SocketAddr>,
    pending: HashSet<SocketAddr>,
    shutdown: bool,
}

impl Swarm {
    pub fn new() -> Self {
        Self {
            candidates: VecDeque::new(),
            actives: HashSet::new(),
            checked: HashSet::new(),
            pending: HashSet::new(),
            shutdown: false,
        }
    }

    pub async fn run(&mut self, swarm_rx: &mut mpsc::Receiver<SwarmEvent>) {
        loop {
            tokio::select! {
                event = swarm_rx.recv() => {
                    match event {
                        Some(SwarmEvent::NewCandidates(candidates)) => self.add_candidates(candidates),
                        Some(SwarmEvent::PeerDisconnected(addr)) => {
                            self.actives.remove(&addr);
                            tracing::debug!("Swarm: {addr} has disconnected leaving the swarm with {} actives and {} candidates.", self.actives.len(), self.candidates.len());

                            self.fill_slots();
                        },
                        Some(SwarmEvent::Shutdown) | None => {
                            self.shutdown = true;
                        },
                    }
                }
            }
        }
    }

    fn add_candidates(&mut self, candidates: Vec<SocketAddr>) {
        for candidate in candidates {
            if self.checked.insert(candidate) {
                self.candidates.push_back(candidate);
            }
        }

        tracing::debug!("Swarm: Added new candidates, the Swarm now has {} candidates in total.", self.candidates.len());
    }

    fn fill_slots(&mut self) {
        if self.shutdown { return; }

        let current = self.pending.len() + self.actives.len();
        let want = TARGET_PEERS.saturating_sub(current);

        for i in 0..want {
            let Some(addr) = self.candidates.pop_front() else { break; };

            if self.actives.contains(&addr) || self.pending.contains(&addr) {
                continue;
            }

            tracing::debug!("Swarm: Connecting to {addr}");
            self.pending.insert(addr);

            tokio::spawn(async move {
            });
        }
    }
}

fn connect_and_handshake() {
}
