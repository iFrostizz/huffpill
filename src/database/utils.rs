use structopt::StructOpt;

#[derive(sqlx::FromRow, Debug)]
pub struct UserInfo {
    pub name: String,
    pub port_in: u16,
    pub port_out: u16,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Challenge {
    pub difficulty: u8,
    pub solves: u16,
    pub kind: Vec<ChallengeType>,
}

#[derive(Debug)]
pub enum ChallengeType {
    Puzzle,
    Challenge,
    Gas,
    Size,
}

#[derive(StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(StructOpt)]
pub enum Command {
    Reset {},
    Start {},
    Add { description: String },
    Done { id: i64 },
}
