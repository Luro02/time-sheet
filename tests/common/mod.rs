use time_sheet::input::json_input::MonthFile;
use time_sheet::input::toml_input;
use time_sheet::input::Config;
use time_sheet::time::WorkingDuration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexMap<K, V> {
    inner: Vec<(K, V)>,
}

impl<K, V> IndexMap<K, V> {
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    fn find_key(&self, key: &K) -> Option<usize>
    where
        K: Eq,
    {
        self.inner
            .iter()
            .enumerate()
            .find_map(|(i, (k, _))| (k == key).then(|| i))
    }

    #[must_use]
    pub fn get_mut_or_insert(&mut self, key: K, value: V) -> &mut V
    where
        K: Eq,
    {
        let index = {
            if let Some(index) = self.find_key(&key) {
                index
            } else {
                self.inner.push((key, value));
                self.inner.len() - 1
            }
        };

        &mut self.inner[index].1
    }
}

impl<K, V> From<Vec<(K, V)>> for IndexMap<K, V> {
    fn from(inner: Vec<(K, V)>) -> Self {
        Self { inner }
    }
}

#[must_use]
#[allow(dead_code)]
pub fn get_proportions(json_month_file: &MonthFile) -> IndexMap<&str, WorkingDuration> {
    let mut map: IndexMap<_, WorkingDuration> = IndexMap::new();
    for entry in json_month_file.entries() {
        let value = map.get_mut_or_insert(entry.action(), Default::default());
        *value += entry.work_duration();
    }

    map
}

#[must_use]
pub fn make_month_file(global: toml_input::Global, month: toml_input::Month) -> MonthFile {
    let config = Config::try_from_toml(month, global)
        .expect("config should be valid")
        .build();

    let json_month_file: MonthFile = serde_json::from_str(
        &config
            .to_month_json()
            .expect("should be able to make a json"),
    )
    .expect("should be able to parse the json to a MonthFile");

    json_month_file
}

pub fn make_global(working_time: WorkingDuration) -> String {
    format!(
        concat!(
            //
            "[about]\n",
            "name = \"John Smith\"\n",
            "staff_id = 1234567\n",
            "\n",
            "[contract.MENSA]\n",
            "working_time = \"{working_time}\"\n",
            "area = \"gf\"\n",
            "wage = 12.00\n",
            "start_date = 2009-10-01\n",
            "end_date = 2239-09-30\n",
            "\n",
        ),
        working_time = working_time
    )
}

#[allow(dead_code)]
pub fn debug_setup() {
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_APP_LOG", "trace");
    color_backtrace::install();
    pretty_env_logger::init_custom_env("RUST_APP_LOG");
}
