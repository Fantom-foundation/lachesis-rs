use std::time::Duration;

use std::io;

use actix::prelude::*;

pub struct Heartbeat {
    pub count: usize,
}

impl Heartbeat {
    fn beat(&mut self, _context: &mut Context<Self>) {
        self.count += 1;
    }
}

pub struct GetHeartbeatCount;

impl Message for GetHeartbeatCount {
    type Result = Result<usize, io::Error>;
}

impl Actor for Heartbeat {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        IntervalFunc::new(Duration::new(3, 0), Self::beat)
            .finish()
            .spawn(ctx);
    }
}

impl Handler<GetHeartbeatCount> for Heartbeat {
    type Result = Result<usize, io::Error>;

    fn handle(&mut self, _msg: GetHeartbeatCount, _ctx: &mut Context<Self>) -> Self::Result {
        Ok(self.count)
    }
}
