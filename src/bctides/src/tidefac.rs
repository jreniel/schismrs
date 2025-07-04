use chrono::DateTime;
use chrono::Datelike;
use chrono::Duration;
use chrono::Timelike;
use chrono::Utc;
use lazy_static::lazy_static;
use linked_hash_map::LinkedHashMap;
use std::f64::consts::PI;
lazy_static! {
    static ref TIDAL_SPECIES_TYPE_MAP: LinkedHashMap<&'static str, u8> =
        LinkedHashMap::from_iter([
            ("M2", 2),
            ("S2", 2),
            ("N2", 2),
            ("K2", 2),
            ("K1", 1),
            ("O1", 1),
            ("P1", 1),
            ("Q1", 1),
            ("Z0", 0),
        ]);
}

lazy_static! {
    static ref TIDAL_POTENTIAL_AMPLITUDES_MAP: LinkedHashMap<&'static str, f64> =
        LinkedHashMap::from_iter([
            ("M2", 0.242334),
            ("S2", 0.112841),
            ("N2", 0.046398),
            ("K2", 0.030704),
            ("K1", 0.141565),
            ("O1", 0.100514),
            ("P1", 0.046843),
            ("Q1", 0.019256),
            ("Z0", 0.),
        ]);
}
lazy_static! {
    static ref ORBITAL_FREQUENCIES: LinkedHashMap<&'static str, f64> = LinkedHashMap::from_iter([
        ("M4", 0.0002810378050173),
        ("M6", 0.0004215567080107),
        ("MK3", 0.0002134400613513),
        ("S4", 0.0002908882086657),
        ("MN4", 0.0002783986019952),
        ("S6", 0.0004363323129986),
        ("M3", 0.0002107783537630),
        ("2MK3", 0.0002081166466594),
        ("M8", 0.0005620756090649),
        ("MS4", 0.0002859630068415),
        ("M2", 0.0001405189025086),
        ("S2", 0.0001454441043329),
        ("N2", 0.0001378796994865),
        ("Nu2", 0.0001382329037065),
        ("MU2", 0.0001355937006844),
        ("2N2", 0.0001352404964644),
        ("lambda2", 0.0001428049013108),
        ("T2", 0.0001452450073529),
        ("R2", 0.0001456432013128),
        ("2SM2", 0.0001503693061571),
        ("L2", 0.0001431581055307),
        ("K2", 0.0001458423172006),
        ("K1", 0.0000729211583579),
        ("O1", 0.0000675977441508),
        ("OO1", 0.0000782445730498),
        ("S1", 0.0000727220521664),
        ("M1", 0.0000702594512543),
        ("J1", 0.0000755603613800),
        ("RHO", 0.0000653117453487),
        ("Q1", 0.0000649585411287),
        ("2Q1", 0.0000623193381066),
        ("P1", 0.0000725229459750),
        ("Mm", 0.0000026392030221),
        ("Ssa", 0.0000003982128677),
        ("Sa", 0.0000001991061914),
        ("Msf", 0.0000049252018242),
        ("Mf", 0.0000053234146919),
        ("Z0", 0.0),
    ]);
}

#[derive(Debug)]
pub struct Tidefac<'a> {
    start_date: &'a DateTime<Utc>,
    run_duration: &'a Duration,
    constituent: &'a str,
    // tidal_species_type: &'a u8,
    // tidal_potential_amplitude: &'a f64,
    // orbital_frequency: &'a f64,
}

impl<'a> Tidefac<'a> {
    pub fn start_date(&self) -> &DateTime<Utc> {
        self.start_date
    }
    pub fn run_duration(&self) -> &Duration {
        self.run_duration
    }
    pub fn constituent(&self) -> &str {
        self.constituent
    }
    pub fn tidal_species_type(&self) -> &u8 {
        &TIDAL_SPECIES_TYPE_MAP
            .get(self.constituent)
            .expect(&format!(
                "Failed to get tidal_species_type for constituent {}",
                self.constituent
            ))
    }
    pub fn tidal_potential_amplitude(&self) -> &f64 {
        &TIDAL_POTENTIAL_AMPLITUDES_MAP
            .get(self.constituent)
            .expect(&format!(
                "Failed to get tidal_potential_amplitude for constituent {}",
                self.constituent
            ))
    }
    pub fn orbital_frequency(&self) -> &f64 {
        &ORBITAL_FREQUENCIES.get(self.constituent).expect(&format!(
            "Failed to fetch orbital frequency for contituent: {}",
            self.constituent,
        ))
    }

    pub fn nodal_factor(&self) -> f64 {
        match self.constituent {
            "M2" => self.EQ78(),
            "S2" => 1.,
            "N2" => self.EQ78(),
            "K1" => self.EQ227(),
            "M4" => self.EQ78().powi(2),
            "O1" => self.EQ75(),
            "M6" => self.EQ78().powi(3),
            "MK3" => self.EQ78() * self.EQ227(),
            "S4" => 1.0,
            "MN4" => self.EQ78().powi(2),
            "Nu2" => self.EQ78(),
            "S6" => 1.0,
            "MU2" => self.EQ78(),
            "2N2" => self.EQ78(),
            "OO1" => self.EQ77(),
            "lambda2" => self.EQ78(),
            "S1" => 1.0,
            "M1" => self.EQ207(),
            "J1" => self.EQ76(),
            "Mm" => self.EQ73(),
            "Ssa" => 1.0,
            "Sa" => 1.0,
            "Msf" => self.EQ78(),
            "Mf" => self.EQ74(),
            "RHO" => self.EQ75(),
            "Q1" => self.EQ75(),
            "T2" => 1.0,
            "R2" => 1.0,
            "2Q1" => self.EQ75(),
            "P1" => 1.0,
            "2SM2" => self.EQ78(),
            "M3" => self.EQ149(),
            "L2" => self.EQ215(),
            "2MK3" => self.EQ227() * self.EQ78().powi(2),
            "K2" => self.EQ235(),
            "M8" => self.EQ78().powi(4),
            "MS4" => self.EQ78(),
            "Z0" => 1.,
            _ => panic!("Unhandled constituent: {}", self.constituent),
        }
    }
    pub fn greenwich_factor(&self) -> f64 {
        let result = match self.constituent {
            "M2" => 2.0 * (self.DT() - self.DS() + self.DH()) + 2.0 * (self.DXI() - self.DNU()),
            "S2" => 2.0 * self.DT(),
            "N2" => {
                2.0 * (self.DT() + self.DH()) - 3.0 * self.DS()
                    + self.DP()
                    + 2.0 * (self.DXI() - self.DNU())
            }
            "K1" => self.DT() + self.DH() - 90.0 - self.DNUP(),
            "M4" => 4.0 * (self.DT() - self.DS() + self.DH()) + 4.0 * (self.DXI() - self.DNU()),
            "O1" => self.DT() - 2.0 * self.DS() + self.DH() + 90.0 + 2.0 * self.DXI() - self.DNU(),
            "M6" => 6.0 * (self.DT() - self.DS() + self.DH()) + 6.0 * (self.DXI() - self.DNU()),
            "MK3" => {
                3.0 * (self.DT() + self.DH()) - 2.0 * self.DS() - 90.0
                    + 2.0 * (self.DXI() - self.DNU())
                    - self.DNUP()
            }
            "S4" => 4.0 * self.DT(),
            "MN4" => {
                4.0 * (self.DT() + self.DH()) - 5.0 * self.DS()
                    + self.DP()
                    + 4.0 * (self.DXI() - self.DNU())
            }
            "Nu2" => {
                2.0 * self.DT() - 3.0 * self.DS() + 4.0 * self.DH() - self.DP()
                    + 2.0 * (self.DXI() - self.DNU())
            }
            "S6" => 6.0 * self.DT(),
            "MU2" => {
                2.0 * (self.DT() + 2.0 * (self.DH() - self.DS())) + 2.0 * (self.DXI() - self.DNU())
            }
            "2N2" => {
                2.0 * (self.DT() - 2.0 * self.DS() + self.DH() + self.DP())
                    + 2.0 * (self.DXI() - self.DNU())
            }
            "OO1" => self.DT() + 2.0 * self.DS() + self.DH() - 90.0 - 2.0 * self.DXI() - self.DNU(),
            "lambda2" => {
                2.0 * self.DT() - self.DS() + self.DP() + 180.0 + 2.0 * (self.DXI() - self.DNU())
            }
            "S1" => self.DT(),
            "M1" => self.DT() - self.DS() + self.DH() - 90.0 + self.DXI() - self.DNU() + self.DQ(),
            "J1" => self.DT() + self.DS() + self.DH() - self.DP() - 90.0 - self.DNU(),
            "Mm" => self.DS() - self.DP(),
            "Ssa" => 2.0 * self.DH(),
            "Sa" => self.DH(),
            "Msf" => 2.0 * (self.DS() - self.DH()),
            "Mf" => 2.0 * self.DS() - 2.0 * self.DXI(),
            "RHO" => {
                self.DT() + 3.0 * (self.DH() - self.DS()) - self.DP() + 90.0 + 2.0 * self.DXI()
                    - self.DNU()
            }
            "Q1" => {
                self.DT() - 3.0 * self.DS() + self.DH() + self.DP() + 90.0 + 2.0 * self.DXI()
                    - self.DNU()
            }
            "T2" => 2.0 * self.DT() - self.DH() + self.DP1(),
            "R2" => 2.0 * self.DT() + self.DH() - self.DP1() + 180.0,
            "2Q1" => {
                self.DT() - 4.0 * self.DS() + self.DH() + 2.0 * self.DP() + 90.0 + 2.0 * self.DXI()
                    - self.DNU()
            }
            "P1" => self.DT() - self.DH() + 90.0,
            "2SM2" => 2.0 * (self.DT() + self.DS() - self.DH()) + 2.0 * (self.DNU() - self.DXI()),
            "M3" => 3.0 * (self.DT() - self.DS() + self.DH()) + 3.0 * (self.DXI() - self.DNU()),
            "L2" => {
                2.0 * (self.DT() + self.DH()) - self.DS() - self.DP()
                    + 180.0
                    + 2.0 * (self.DXI() - self.DNU())
                    - self.DR()
            }
            "2MK3" => {
                3.0 * (self.DT() + self.DH()) - 4.0 * self.DS()
                    + 90.0
                    + 4.0 * (self.DXI() - self.DNU())
                    + self.DNUP()
            }
            "K2" => 2.0 * (self.DT() + self.DH()) - 2.0 * self.DNUP2(),
            "M8" => 8.0 * (self.DT() - self.DS() + self.DH()) + 8.0 * (self.DXI() - self.DNU()),
            "MS4" => {
                2.0 * (2.0 * self.DT() - self.DS() + self.DH()) + 2.0 * (self.DXI() - self.DNU())
            }
            "Z0" => 0.0,
            _ => panic!("Unrecognized constituent {}", self.constituent),
        };
        result % 360.
    }

    fn hour_middle(&self) -> f64 {
        let start_hour = self.start_date.hour() as f64;
        let duration_in_hours = self.run_duration.num_seconds() as f64 / 3600.0;
        start_hour + (duration_in_hours / 2.0)
    }

    fn get_lunar_node(&self) -> f64 {
        259.1560564
            - 19.328185764 * self.DYR()
            - 0.0529539336 * (self.DDAY() as f64)
            - 0.0022064139 * self.hour_middle()
    }

    fn get_lunar_perigee(&self) -> f64 {
        334.3837214
            + 40.66246584 * self.DYR()
            + 0.111404016 * (self.DDAY() as f64)
            + 0.004641834 * self.hour_middle()
    }
    fn get_lunar_mean_longitude(&self) -> f64 {
        277.0256206
            + 129.38482032 * self.DYR()
            + 13.176396768 * (self.DDAY() as f64)
            + 0.549016532 * self.start_date().hour() as f64
    }

    fn get_solar_perigee(&self) -> f64 {
        281.2208569
            + 0.01717836 * self.DYR()
            + 0.000047064 * (self.DDAY() as f64)
            + 0.000001961 * self.start_date().hour() as f64
    }

    fn get_solar_mean_longitude(&self) -> f64 {
        280.1895014 - 0.238724988 * self.DYR()
            + 0.9856473288 * (self.DDAY() as f64)
            + 0.0410686387 * self.start_date().hour() as f64
    }
    #[allow(non_snake_case)]
    fn I(&self) -> f64 {
        (0.9136949 - 0.0356926 * self.N().cos()).acos()
    }

    fn deg_to_rad(degrees: f64) -> f64 {
        degrees * (PI / 180.0)
    }

    #[allow(non_snake_case)]
    fn N(&self) -> f64 {
        Self::deg_to_rad(self.DN())
    }

    #[allow(non_snake_case)]
    fn DN(&self) -> f64 {
        self.get_lunar_node()
    }

    #[allow(non_snake_case)]
    fn EQ73(&self) -> f64 {
        (2. / 3. - self.I().sin().powi(2)) / 0.5021
    }

    #[allow(non_snake_case)]
    fn EQ74(&self) -> f64 {
        (self.I().sin()).powi(2) / 0.1578
    }

    #[allow(non_snake_case)]
    fn EQ75(&self) -> f64 {
        self.I().sin() * (self.I() / 2.).cos().powi(2) / 0.37988
    }

    #[allow(non_snake_case)]
    fn EQ76(&self) -> f64 {
        (2.0 * self.I()).sin() / 0.7214
    }

    #[allow(non_snake_case)]
    fn EQ77(&self) -> f64 {
        self.I().sin() * (self.I() / 2.0).sin().powi(2) / 0.0164
    }

    #[allow(non_snake_case)]
    fn EQ78(&self) -> f64 {
        ((self.I() / 2.).cos()).powi(4) / 0.91544
    }

    #[allow(non_snake_case)]
    fn EQ149(&self) -> f64 {
        (self.I() / 2.0).cos().powi(6) / 0.8758
    }

    #[allow(non_snake_case)]
    fn EQ197(&self) -> f64 {
        (2.310 + 1.435 * (2.0 * (self.P() - self.XI())).cos()).sqrt()
    }

    #[allow(non_snake_case)]
    fn EQ207(&self) -> f64 {
        self.EQ75() * self.EQ197()
    }

    #[allow(non_snake_case)]
    fn EQ213(&self) -> f64 {
        (1.0 - 12.0 * (self.I() / 2.0).tan().powi(2) * (2.0 * self.P()).cos()
            + 36.0 * (self.I() / 2.0).tan().powi(4))
        .sqrt()
    }

    #[allow(non_snake_case)]
    fn EQ215(&self) -> f64 {
        self.EQ78() * self.EQ213()
    }

    #[allow(non_snake_case)]
    fn EQ227(&self) -> f64 {
        (0.8965 * (2. * self.I()).sin().powi(2)
            + 0.6001 * (2. * self.I()).sin() * self.NU().cos()
            + 0.1006)
            .sqrt()
    }

    #[allow(non_snake_case)]
    fn EQ235(&self) -> f64 {
        0.001
            + (19.0444 * self.I().sin().powi(4)
                + 2.7702 * self.I().sin().powi(2) * (2.0 * self.NU()).cos()
                + 0.0981)
                .sqrt()
    }
    #[allow(non_snake_case)]
    fn DYR(&self) -> f64 {
        self.start_date.year() as f64 - 1900.
    }

    #[allow(non_snake_case)]
    fn DDAY(&self) -> i32 {
        let day_of_year = self.start_date.ordinal() as i32;
        let years_since_1901 = self.start_date.year() - 1901;
        let leap_years_since_1901 = ((years_since_1901 - 1) / 4) as i32;
        day_of_year + leap_years_since_1901 - 1
    }
    #[allow(non_snake_case)]
    fn NU(&self) -> f64 {
        0.0897056 * self.N().sin() / self.I().sin().asin()
    }

    #[allow(non_snake_case)]
    fn DT(&self) -> f64 {
        180.0 + self.start_date().hour() as f64 * (360.0 / 24.0)
    }

    #[allow(non_snake_case)]
    fn DS(&self) -> f64 {
        self.get_lunar_mean_longitude()
    }

    #[allow(non_snake_case)]
    fn DP(&self) -> f64 {
        self.get_lunar_perigee()
    }

    #[allow(non_snake_case)]
    fn P(&self) -> f64 {
        self.DP().to_radians()
    }

    #[allow(non_snake_case)]
    fn DH(&self) -> f64 {
        self.get_solar_mean_longitude()
    }

    #[allow(non_snake_case)]
    fn DP1(&self) -> f64 {
        self.get_solar_perigee()
    }

    #[allow(non_snake_case)]
    fn DNU(&self) -> f64 {
        self.NU().to_degrees()
    }

    #[allow(non_snake_case)]
    fn XI(&self) -> f64 {
        self.N() - 2.0 * (0.64412 * self.N() / 2.0).tan().atan() - self.NU()
    }

    #[allow(non_snake_case)]
    fn DXI(&self) -> f64 {
        self.XI().to_degrees()
    }

    #[allow(non_snake_case)]
    fn NUP(&self) -> f64 {
        (self.NU().sin() / (self.NU().cos() + 0.334766 / (2.0 * self.I()).sin())).atan()
    }

    #[allow(non_snake_case)]
    fn DNUP(&self) -> f64 {
        self.NUP().to_degrees()
    }

    #[allow(non_snake_case)]
    fn DPC(&self) -> f64 {
        self.DP() - self.DXI()
    }

    #[allow(non_snake_case)]
    fn PC(&self) -> f64 {
        self.DPC().to_radians()
    }

    #[allow(non_snake_case)]
    fn R(&self) -> f64 {
        (2.0 * self.PC()).sin().atan()
            / ((1.0 / 6.0) * (0.5 * self.I()).tan().recip().powi(2) - (2.0 * self.PC()).cos())
    }

    #[allow(non_snake_case)]
    fn DR(&self) -> f64 {
        self.R().to_degrees()
    }

    #[allow(non_snake_case)]
    fn NUP2(&self) -> f64 {
        ((2.0 * self.NU()).sin() / ((2.0 * self.NU()).cos() + 0.0726184 / self.I().sin().powi(2)))
            .atan()
            / 2.0
    }

    #[allow(non_snake_case)]
    fn DNUP2(&self) -> f64 {
        self.NUP2().to_degrees()
    }

    #[allow(non_snake_case)]
    fn Q(&self) -> f64 {
        (5.0 * self.I().cos() - 1.0).atan2((7.0 * self.I().cos() + 1.0) * self.PC().cos())
            * self.PC().sin()
    }

    #[allow(non_snake_case)]
    fn DQ(&self) -> f64 {
        self.Q().to_degrees()
    }
}

pub fn tidefac<'a>(
    start_date: &'a DateTime<Utc>,
    run_duration: &'a Duration,
    constituent: &'a str,
) -> Tidefac<'a> {
    // TidefacBuilder::default()
    //     .start_date(start_date)
    //     .run_duration(run_duration)
    //     .build()
    //     .unwrap()
    Tidefac {
        start_date,
        run_duration,
        constituent: constituent.strip_prefix('_').unwrap_or(constituent),
        // tidal_species_type: &TIDAL_SPECIES_TYPE_MAP.get(constituent).unwrap(),
        // tidal_potential_amplitude: &TIDAL_POTENTIAL_AMPLITUDES_MAP.get(constituent).unwrap(),
        // orbital_frequency: &ORBITAL_FREQUENCIES.get(constituent).unwrap(),
    }
}
