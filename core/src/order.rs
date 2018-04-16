use events::Events;
// we process these
use events::OrderEvent;
// we emit these
use events::ReceiveEvent::GotMessage as R_GotMessage;
use events::KeyEvent::GotPake as K_GotPake;

#[derive(Debug, PartialEq)]
enum State {
    S0, //no pake
    S1, //yes pake
}

pub struct Order {
    state: State,
    queue: Vec<(String, String, String)>,
}

enum QueueStatus {
    Enqueue((String, String, String)),
    Drain,
    NoAction,
}

impl Order {
    pub fn new() -> Order {
        Order {
            state: State::S0,
            queue: Vec::new(),
        }
    }

    pub fn process(&mut self, event: OrderEvent) -> Events {
        use self::State::*;

        println!(
            "order: current state = {:?}, got event = {:?}",
            self.state, event
        );

        let (newstate, actions, queue_status) = match self.state {
            S0 => self.do_S0(event),
            S1 => self.do_S1(event),
        };

        self.state = newstate;

        match queue_status {
            QueueStatus::Enqueue(tup) => self.queue.push(tup),
            QueueStatus::Drain => {
                self.queue = Vec::new();
            }
            QueueStatus::NoAction => (),
        };

        actions
    }

    fn drain(&self) -> Events {
        let mut es = Events::new();

        for &(ref side, ref phase, ref body) in &self.queue {
            es.push(R_GotMessage(
                side.to_string(),
                phase.to_string(),
                body.as_bytes().to_vec(),
            ));
        }

        es
    }

    fn do_S0(&self, event: OrderEvent) -> (State, Events, QueueStatus) {
        use events::OrderEvent::*;
        match event {
            GotMessage(side, phase, body) => {
                if phase == "pake" {
                    // got a pake message
                    println!("key: got pake message at S0");
                    let mut es = self.drain();
                    let mut key_events = events![K_GotPake(body)];
                    key_events.append(&mut es);
                    (State::S1, key_events, QueueStatus::Drain) // TODO: instead of None, return a queue status
                } else {
                    // not a  pake message, queue it.
                    println!("key: not a pake message. Queue it (S0)");
                    (
                        State::S0,
                        events![],
                        QueueStatus::Enqueue((side, phase, body)),
                    )
                }
            }
            _ => panic!(),
        }
    }

    fn do_S1(&self, event: OrderEvent) -> (State, Events, QueueStatus) {
        use events::OrderEvent::*;
        match event {
            GotMessage(side, phase, body) => (
                State::S1,
                events![R_GotMessage(side, phase, body.as_bytes().to_vec())],
                QueueStatus::NoAction,
            ),
        }
    }
}
