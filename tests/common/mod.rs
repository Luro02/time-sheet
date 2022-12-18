use time_sheet::time::WorkingDuration;

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
