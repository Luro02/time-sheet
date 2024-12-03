time-sheet
===

Spend more time on the important things.


For jobs at [KIT](https://www.kit.edu/) one has to submit a time-sheet each month, detailing when and what has been worked on.
Keeping track of everything (like repeating events) and when exactly one worked, can be time consuming.

This App is built on top of [TimeSheetGenerator](https://github.com/kit-sdq/TimeSheetGenerator) that generates the final PDF and validates that the config is valid.

## Features
- [x] automatically add repeating events like regular meetings
- [x] automatically add and calculate **holidays**
- [x] support entries that have no fixed time, used to fill the sheet until required time is reached
- [x] add signature to the document
- [x] change the background header to the correct department
- [x] send the sheet via email

## Compiling

First clone the repository, keep in mind that this has to be done recursively, because of the submodules:

```
$ git clone https://github.com/Luro02/time-sheet --recursive
```

This program is written in [rust](https://www.rust-lang.org/) and uses nightly features. You have to install the nightly toolchain first.

To do this, first download [`rustup`](https://rustup.rs/).

After installing it, you can add the nightly toolchain by running this command:
```
$ rustup install nightly
```

Then you can compile the project by running
```
$ cargo +nightly build --release
```

The binary will be in `target/release/time-sheet.exe`.

Alternatively, you can execute the program without first building, with this command:
```
$ cargo +nightly run --release -- <here the arguments>
```
for example like this
```
$ cargo +nightly run --release -- make --global path/to/global.toml --month path/to/month/02.toml
```

For execution you have to make sure that the files in `resources/` are present
and the following commands have to work:
- `java`
- `latexmk` (I installed [this one](https://miktex.org/download))

## Example: Math Tutorium

The program requires two files as input.

The `global.toml` contains data that does not change between the months, like name or informations about the contract.

It can look like this
```toml
[about]
name = "Max Mustermann"
staff_id = 1344222

# This section is for general options related that do not fit
# in any other section, like specifying the filenames of generated
# files or whether the temporary folder should not be temporary.
[config]
# By default the generated files will have a filename in the format:
# "{year:04}-{month:02}.pdf", like "2024-01.pdf"
#
# With this option one can specify a different format.
# The following things can be inserted into the format string:
# - last_name (will be parsed from the name field in the about section,
#   make sure that the name is "{first_name} {last_name}")
# - first_name
# - year
# - month
#
# You can add padding to the numbers like in the following example.
output_format = "{last_name}_{first_name}_{month:02}_{year:04}.pdf"

# Here you can manually specify the path to the latexmk binary:
# latex_mk_path = "C:\\path\\to\\latexmk"

# If specified, the program will use that directory for storing
# the temp files.
# preserve_dir = "C:\\path\to\\non-temporary\\directory\\"


[about.signature]
# the path to an image of the signature
path = "D:\\Signature.png"
# optionally one can specify the width
# of the signature in cm on the final
# sheet:
# width = 3.2
# (default value is 3.8)
#
# To find the best value, it is
# recommended to just try out a few.

# contract.<institute/department>
# where one works.
# It is possible to add multiple
# contracts here
[contract.IANA]
# How much one should work in a month
working_time = "40:00"
# gf = großforschungsbereich
# ub = universitätsbereich
#
# for tutoriums the correct value
# would be "ub"
area = "ub"
# How much money on makes each month
wage = 12.00
# In the bottom left of the final pdf
# is this text in the background.
# Through this option the value can
# be configured.
bg_content = "K_IANA_AZDoku_01_01-20"
# When does the contract start?
start_date = 2022-01-01
# When does the contract end?
end_date = 2023-01-01

# A core feature is that one can
# specify repeating events.
#
# For example:

[repeating."Tutorium halten"]
start = "09:35"
end = "11:15"
repeats_on = ["Tuesday"]
repeats_every = "week"
department = "IANA"

[repeating."Tutoren Besprechung"]
start = "10:00"
end = "11:00"
repeats_on = ["Friday"]
repeats_every = "week"
department = "IANA"
```

The `month.toml` contains data that is specific to a single month, like when one has worked.

It can look like this
```toml
[general]
month = 12
year = 2022
department = "IANA"

# here one can specify the date under
# the signature (when the document
# has been signed)
[general.signature]
date = "2022-12-31"

# By adding this section holidays
# will be added to the month.
#
# As far as I know you are required
# to take holidays, so it is better
# to add them like below:
[holiday]
# the day where one takes the holiday
day = 23
# starting time (does not really
# matter, when no other work is done
# on that day)
start = "11:00"
# for how many months the holiday is
# taken. At the end of the year, you
# should have had 12 months of
# holidays.
#
# So for example one could not have
# holidays in january, february and
# march and then take 3 months of
# holidays in april.
months = 1

# Sometimes one works too much in a
# month. Through this section one
# can specify by how much one exceeded
# the working time last month or by
# how much one will exceed the working
# time in the next month.
[transfer]
previous_month = "0:00"
next_month = "0:00"

# Specify that no repeating or dynamic
# event can happen in that time.
#
# In this case, because of semester
# holidays.
[absence.24-31]
start = "00:00"
end = "23:59"

# One can declare entries explicitly
# like this (13th december 2022):
[entries.13]
action = "Besprechung"
start = "13:10"
end = "14:30"

[entries.15]
action = "Korrektur"
start = "12:00"
end = "18:00"
pause = "01:00"

[entries.16]
action = "Korrektur"
start = "12:00"
end = "15:00"
# signal that of the 3 hours
# 1 hour was pause (no work)
#
# pauses are not paid for, but
# one has to specify them if one
# has worked more than x hours
# on a single day
pause = "01:00"

# This is another one of the main
# features.
#
# As can be seen no date is specified
# of when this work has been done.
#
# The dates will be decided by this
# program automatically and it will
# only add dates until the 40:00h are
# reached for the month.
[dynamic."Tutorium vorbereiten"]
duration = "40:00"
```

One can create a pdf by running
```
$ time-sheet make --global global.toml --month 12.toml
```
The PDF will be saved here `pdfs/12.pdf`.

## Sending an E-Mail

To send an email, one can use the `send` command:
```
$ time-sheet send --global global.toml --month 12.toml --subject "Max Mustermann {year:02}-{month:02}" max@kit.edu
```

The email will not have any text, only the file as an attachment.
