#[derive(Deserialize, Default)]
pub struct OptionSet {
    pub port: Option<u16>,
    pub ranking: Option<String>,
    pub load_mission_to_memory: Option<bool>,
    pub bandwidth_algorithm: Option<u8>,
    pub cpu_count: Option<i8>,
    pub ex_threads: Option<u8>,
    pub enable_ht: Option<bool>,
    pub hugepages: Option<bool>,
    pub auto_init: Option<bool>,

    pub basic: Option<String>,
    pub config: Option<String>,
    pub profile: Option<String>,
    pub modpack: Option<String>,
    pub server_modpack: Option<String>,
}

macro_rules! merge {
    ($s:expr, $o:expr, $a:ident) => {
        $s.$a.clone().or($o.$a)
    };
}

impl OptionSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: Self) {
        *self = Self {
            port: merge!(self, other, port).or(Some(2302)),
            ranking: merge!(self, other, ranking),
            load_mission_to_memory: merge!(self, other, load_mission_to_memory).or(Some(true)),
            bandwidth_algorithm: merge!(self, other, bandwidth_algorithm).or(Some(2)),
            cpu_count: merge!(self, other, cpu_count).map(|x| {
                if x.is_negative() {
                    num_cpus::get() as i8
                } else {
                    x
                }
            }),
            ex_threads: merge!(self, other, ex_threads),
            enable_ht: merge!(self, other, enable_ht),
            hugepages: merge!(self, other, hugepages),
            auto_init: merge!(self, other, auto_init),

            basic: merge!(self, other, basic),
            config: merge!(self, other, config),
            profile: merge!(self, other, profile),
            modpack: merge!(self, other, modpack),
            server_modpack: merge!(self, other, server_modpack),
        }
    }
}
