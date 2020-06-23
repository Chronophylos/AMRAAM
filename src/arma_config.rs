use anyhow::Result;
use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, tag_no_case, take_till, take_while},
    character::complete::{alphanumeric1, char, digit1, line_ending, not_line_ending, space0},
    combinator::{all_consuming, map, map_res, opt},
    multi::{fold_many0, many0, separated_list},
    number::complete::double,
    sequence::{separated_pair, terminated, tuple},
    IResult,
};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Number(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<Self>),
    Class(HashMap<String, Value>),
}

#[derive(Debug, PartialEq)]
pub struct Cfg(Value);

fn string(i: &str) -> IResult<&str, Value> {
    let (i, _) = char('"')(i)?;
    map(take_till(|c| c == '"'), |x: &str| {
        Value::String(x.to_owned())
    })(i)
}

fn number(i: &str) -> IResult<&str, Value> {
    map(
        map_res(tuple((opt(char('-')), digit1)), |(sign, s)| {
            i64::from_str_radix(&format!("{}{}", sign.unwrap_or('+'), s), 10)
        }),
        Value::Number,
    )(i)
}

fn float(i: &str) -> IResult<&str, Value> {
    map(double, Value::Float)(i)
}

fn boolean(i: &str) -> IResult<&str, Value> {
    map(
        map_res(alt((tag_no_case("true"), tag_no_case("false"))), |t| {
            bool::from_str(t)
        }),
        Value::Boolean,
    )(i)
}

fn list_separator(i: &str) -> IResult<&str, ()> {
    let (i, _) = space0(i)?;
    let (i, _) = char(',')(i)?;
    let (i, _) = space0(i)?;

    Ok((i, ()))
}

fn list(i: &str) -> IResult<&str, Value> {
    let (i, _) = char('{')(i)?;
    let (i, v) = map(separated_list(list_separator, value_in_list), Value::List)(i)?;
    let (i, _) = char('{')(i)?;

    Ok((i, v))
}

fn class(i: &str) -> IResult<&str, (&str, Value)> {
    let (i, (_, _, _, klass, _, map, _)) = tuple((
        space0,
        tag("class"),
        space0,
        alphanumeric1,
        char('{'),
        parser,
        tag("};"),
    ))(i)?;

    Ok((i, (klass, Value::Class(map))))
}

fn value_in_list(i: &str) -> IResult<&str, Value> {
    alt((string, number, float, boolean))(i)
}

fn value(i: &str) -> IResult<&str, Value> {
    alt((string, number, float, boolean, list, class))(i)
}

fn equals(i: &str) -> IResult<&str, (&str, char, &str)> {
    tuple((space0, char('='), space0))(i)
}

fn key(i: &str) -> IResult<&str, &str> {
    unimplemented!();
}

fn kv_end(i: &str) -> IResult<&str, char> {
    char(';')(i)
}

fn comment(i: &str) -> IResult<&str, String> {
    let (i, _) = tag("//")(i)?;
    fold_many0(not_line_ending, String::new(), |mut acc, x| {
        acc.push_str(x);
        acc
    })(i)
}

fn key_value(i: &str) -> IResult<&str, (&str, Value)> {
    terminated(separated_pair(key, equals, value), kv_end)(i)
}

fn line(i: &str) -> IResult<&str, (Option<(&str, Value)>, Option<String>)> {
    tuple((opt(key_value), opt(comment)))(i)
}

fn parser(i: &str) -> IResult<&str, HashMap<String, Value>> {
    fold_many0(line, HashMap::new(), |mut acc, (kv, _)| {
        if let Some((key, value)) = kv {
            acc.insert(key.to_string(), value);
        };
        acc
    })(i)
}

impl Cfg {
    pub fn from_string(s: &'static str) -> Result<Self> {
        let (_, map) = all_consuming(parser)(s)?;
        Ok(Self(Value::Class(map)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_full() {
        let mut map = HashMap::new();
        map.insert("foo".into(), Value::String("bar".into()));
        map.insert("the_answer".into(), Value::Number(42));
        map.insert("pi".into(), Value::Float(3.14));
        map.insert(
            "emotes".into(),
            Value::List(vec![
                Value::String("peepoStare".into()),
                Value::String("monkaHellascared".into()),
            ]),
        );
        map.insert("working".into(), Value::Boolean(false));

        let mut map2 = HashMap::new();
        map2.insert("subkey".into(), Value::Number(1));

        map.insert("table".into(), Value::Class(map2));

        let config = Cfg(Value::Class(map));

        let s = r#"
        //a aaa
foo = "bar";
pi = 3.14;
        the_answer=42;
emotes[] = {"peepoStare", // break everything
"monkaHellascared"};
class table    {subkey=1};
// comment
working        = false;
"#;
    }

    fn test_real() {
        let server_config = r#"
//
// server.cfg
//
// comments are written with "//" in front of them.

// NOTE: More parameters and details are available at http://community.bistudio.com/wiki/server.cfg

// STEAM PORTS (not needed anymore, it's +1 +2 to gameport)
// steamPort		= 8766;		// default  8766, needs to be unique if multiple servers are on the same box
// steamQueryPort	= 27016;	// default 27016, needs to be unique if multiple servers are on the same box

// GENERAL SETTINGS
hostname		= "My Arma 3 Server";	// Name of the server displayed in the public server list
//password		= "ServerPassword";		// Password required to join the server (remove // at start of line to enable)
passwordAdmin	= "AdminPassword";		// Password to login as admin. Open the chat and type: #login password
maxPlayers		= 40;	// Maximum amount of players, including headless clients. Anybody who joins the server is considered a player, regardless of their role or team.
persistent		= 1;	// If set to 1, missions will continue to run after all players have disconnected; required if you want to use the -autoInit startup parameter

// VOICE CHAT
disableVoN		= 0;	// If set to 1, voice chat will be disabled
vonCodecQuality	= 10;	// Supports range 1-30, the higher the better sound quality, the more bandwidth consumption:
                        // 1-10 is 8kHz (narrowband)
                        // 11-20 is 16kHz (wideband)
                        // 21-30 is 32kHz (ultrawideband)

// VOTING
voteMissionPlayers	= 1;		// Minimum number of players required before displaying the mission selection screen, if you have not already selected a mission in this config
voteThreshold		= 0.33;		// Percentage (0.00 to 1.00) of players needed to vote something into effect, for example an admin or a new mission. Set to 9999 to disable voting.
allowedVoteCmds[] =				// Voting commands allowed to players
{
    // {command, preinit, postinit, threshold} - specifying a threshold value will override "voteThreshold" for that command
    {"admin", false, false},		// vote admin
    {"kick", false, true, 0.51},	// vote kick
    {"missions", false, false},		// mission change
    {"mission", false, false},		// mission selection
    {"restart", false, false},		// mission restart
    {"reassign", false, false}		// mission restart with roles unassigned
};

// WELCOME MESSAGE ("message of the day")
// It can be several lines, separated by comma
// Empty messages "" will not be displayed, but can be used to increase the delay before other messages
motd[] =
{
    "Welcome to My Arma 3 Server",
    "Discord: discord.somewhere.com",
    "TeamSpeak: ts.somewhere.com",
    "Website: www.example.com"
};
motdInterval = 5;	// Number of seconds between each message

// MISSIONS CYCLE
class Missions
{
    class Mission1
    {
        template	= "MyMission.Altis";	// Filename of pbo in MPMissions folder
        difficulty	= "Regular";			// "Recruit", "Regular", "Veteran", "Custom"
    };
};

// LOGGING
timeStampFormat	= "short";				// Timestamp format used in the server RPT logs. Possible values are "none" (default), "short", "full"
logFile			= "server_console.log";	// Server console output filename

// SECURITY
BattlEye				= 1;	// If set to 1, BattlEye Anti-Cheat will be enabled on the server (default: 1, recommended: 1)
verifySignatures		= 2;	// If set to 2, players with unknown or unsigned mods won't be allowed join (default: 0, recommended: 2)
kickDuplicate			= 1;	// If set to 1, players with an ID that is identical to another player will be kicked (recommended: 1)
allowedFilePatching		= 1;	// Prevents clients with filePatching enabled from joining the server
                                // (0 = block filePatching, 1 = allow headless clients, 2 = allow all) (default: 0, recommended: 1)

// FILE EXTENSIONS

// only allow files with those extensions to be loaded via loadFile command (since Arma 3 v1.19.124216)
allowedLoadFileExtensions[] =		{"hpp","sqs","sqf","fsm","cpp","paa","txt","xml","inc","ext","sqm","ods","fxy","lip","csv","kb","bik","bikb","html","htm","biedi"};

// only allow files with those extensions to be loaded via preprocessFile / preprocessFileLineNumbers commands (since Arma 3 v1.19.124323)
allowedPreprocessFileExtensions[] =	{"hpp","sqs","sqf","fsm","cpp","paa","txt","xml","inc","ext","sqm","ods","fxy","lip","csv","kb","bik","bikb","html","htm","biedi"};

// only allow files and URLs with those extensions to be loaded via htmlLoad command (since Arma 3 v1.27.126715)
allowedHTMLLoadExtensions[] =		{"htm","html","php","xml","txt"};

// EVENT SCRIPTS - see http://community.bistudio.com/wiki/ArmA:_Server_Side_Scripting
onUserConnected		= "";	// command to run when a player connects
onUserDisconnected	= "";	// command to run when a player disconnects
doubleIdDetected	= "";	// command to run if a player has the same ID as another player in the server
onUnsignedData		= "kick (_this select 0)";	// command to run if a player has unsigned files
onHackedData		= "kick (_this select 0)";	// command to run if a player has tampered files

// HEADLESS CLIENT
headlessClients[]	= {"127.0.0.1"};	// list of IP addresses allowed to connect using headless clients; example: {"127.0.0.1", "192.168.1.100"};
localClient[]		= {"127.0.0.1"};	// list of IP addresses to which are granted unlimited bandwidth;  example: {"127.0.0.1", "192.168.1.100"};
"#;
        assert_eq!(
            Cfg(Value::Boolean(false)),
            Cfg::from_string(server_config).unwrap()
        );
    }
}
