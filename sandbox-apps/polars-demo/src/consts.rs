use std::cell::LazyCell;
use std::collections::HashMap;

/// The columns we are interested in.
pub const CATEGORICAL_COLS: &[&str] = &["sex", "race", "ethnicity", "education"];
/// Charlson covariates for comorbidity.
pub const CHARLSON_COVARIATES: &[&str] = &[
    "mi", "chf", "pvd", "cevd", "dementia", "copd", "rheumd", "pud", "mld", "msld", "diab",
    "dia_w_c", "hp", "mrend", "srend", "aids", "hiv", "mst", "mal", "Obesity", "WL", "Alcohol",
    "Drug", "Psycho", "Dep",
];
pub const TRAVEL_COVARATES: &[&str] = &["min_travel_time", "travel_time_squared"];

pub const PREDICTORS: &[&str] = &[
    "sex",
    "race",
    "ethnicity",
    "education",
    "income",
    "min_travel_time",
    "SDOH",
];

// Using a macro to make it easier to define the nested HashMap
// This isn't strictly necessary but makes the syntax cleaner than raw HashMap::from
macro_rules! map {
    ($($key:expr => $value:expr),* $(,)?) => {
        std::collections::HashMap::from([
            $((String::from($key), $value)),*
        ])
    };
}

pub const CHARLSON_CONDITIONS: LazyCell<HashMap<String, HashMap<String, Vec<String>>>> =
    LazyCell::new(|| {
        map! {
            "mi" => map! {
                "9" => vec!["410".to_string(), "412".to_string()],
                "10" => vec!["I21".to_string(), "I22".to_string(), "I25.2".to_string()],
            },
            "chf" => map! {
                "9" => vec!["398.91".to_string(), "402.01".to_string(),"402.11".to_string(),"402.91".to_string(),"404.01".to_string(),"404.03".to_string(),"404.11".to_string(),"404.13".to_string(),"404.91".to_string(),"404.93".to_string(),"425.4".to_string(),"425.5".to_string(),"425.6".to_string(),"425.7".to_string(),"425.8".to_string(),"425.9".to_string(),"428".to_string()],
                "10" => vec!["I09.9".to_string(), "I11.0".to_string(), "I13.0".to_string(), "I13.2".to_string(), "I25.5".to_string(), "I42.0".to_string(), "I42.5".to_string(), "I42.6".to_string(), "I42.7".to_string(), "I42.8".to_string(), "I42.9".to_string(), "I43".to_string(), "I50".to_string(), "P29.0".to_string()],
            },
            "pvd" => map! {
                "9" => vec!["093.0".to_string(), "440".to_string(),"441".to_string(),"443.1".to_string(),"443.2".to_string(),"443.3".to_string(),"443.4".to_string(),"443.5".to_string(),"443.6".to_string(),"443.7".to_string(),"443.8".to_string(),"443.9".to_string(),"557.1".to_string(),"557.9".to_string(),"V43.4".to_string()],
                "10" => vec!["I70".to_string(), "I71".to_string(), "I73.1".to_string(),"I73.8".to_string(),"I73.9".to_string(),"I77.1".to_string(),"I79.0".to_string(),"I79.2".to_string(),"K55.1".to_string(),"K55.8".to_string(),"K55.9".to_string(),"Z95.8".to_string(),"Z95.9".to_string()],
            },
            "cevd" => map! {
                "9" => vec!["362.34".to_string(), "430".to_string(),"431".to_string(),"432".to_string(),"433".to_string(),"434".to_string(),"435".to_string(),"436".to_string(),"437".to_string(),"438".to_string()],
                "10" => vec!["G45".to_string(), "G46".to_string(), "H34.0".to_string(),"I60".to_string(), "I61".to_string(), "I62".to_string(),"I63".to_string(),"I64".to_string(),"I65".to_string(),"I66".to_string(),"I67".to_string(),"I68".to_string(),"I69".to_string()],
            },
            "dementia" => map! {
                "9" => vec!["290".to_string(),"294.1".to_string(),"331.2".to_string()],
                "10" => vec!["F00".to_string(),"F01".to_string(),"F02".to_string(),"F03".to_string(),"F05.1".to_string(),"G30".to_string(),"G31.1".to_string()],
            },
            "copd" => map! {
                "9" => vec!["416.8".to_string(),"416.9".to_string(),"490".to_string(), "491".to_string(),"492".to_string(),"493".to_string(),"494".to_string(),"495".to_string(),"496".to_string(),"497".to_string(),"498".to_string(),"499".to_string(),"500".to_string(),"501".to_string(),"502".to_string(),"503".to_string(),"504".to_string(),"505".to_string(),"506.4".to_string(),"508.1".to_string(), "508.8".to_string()],
                "10" => vec!["I27.8".to_string(),"I27.9".to_string(),"J40".to_string(), "J41".to_string(), "J42".to_string(),"J43".to_string(),"J44".to_string(),"J45".to_string(),"J46".to_string(),"J47".to_string(),"J60".to_string(),"J61".to_string(),"J62".to_string(),"J63".to_string(),"J64".to_string(),"J65".to_string(),"J66".to_string(),"J67".to_string(),"J68.4".to_string(),"J70.1".to_string(), "J70.3".to_string()],
            },
            "rheumd" => map! {
                "9" => vec!["446.5".to_string(), "710.0".to_string(), "710.1".to_string(), "710.2".to_string(), "710.3".to_string(), "710.4".to_string(), "714.0".to_string(), "714.1".to_string(), "714.2".to_string(), "714.8".to_string(), "725".to_string()],
                "10" => vec!["M05".to_string(), "M06".to_string(), "M31.5".to_string(), "M32".to_string(), "M33".to_string(), "M34".to_string(), "M35.1".to_string(), "M35.3".to_string(), "M36.0".to_string()],
            },
            "pud" => map! {
                "9" => vec!["531".to_string(), "532".to_string(), "533".to_string(), "534".to_string()],
                "10" => vec!["K25".to_string(), "K26".to_string(), "K27".to_string(), "K28".to_string()],
            },
            "mld" => map! {
                "9" => vec!["070.22".to_string(), "070.23".to_string(), "070.32".to_string(), "070.33".to_string(), "070.44".to_string(), "070.54".to_string(), "070.6".to_string(), "070.9".to_string(), "570".to_string(), "571".to_string(), "573.3".to_string(), "573.4".to_string(), "573.8".to_string(), "573.9".to_string(), "V42.7".to_string()],
                "10" => vec!["B18".to_string(), "K70.0".to_string(), "K70.1".to_string(), "K70.2".to_string(), "K70.3".to_string(), "K70.9".to_string(), "K71.3".to_string(), "K71.4".to_string(), "K71.5".to_string(), "K71.7".to_string(), "K73".to_string(), "K74".to_string(), "K76.0".to_string(), "K76.2".to_string(), "K76.3".to_string(), "K76.4".to_string(), "K76.8".to_string(), "K76.9".to_string(), "Z94.4".to_string()],
            },
            "msld" => map! {
                "9" => vec!["456.0".to_string(), "456.1".to_string(), "456.2".to_string(), "572.2".to_string(), "572.3".to_string(), "572.4".to_string(),"572.5".to_string(),"572.6".to_string(),"572.7".to_string(), "572.8".to_string()],
                "10" => vec!["I85.0".to_string(),"I85.9".to_string(), "I86.4".to_string(),"I98.2".to_string(), "K70.4".to_string(), "K71.1".to_string(), "K72.1".to_string(), "K72.9".to_string(), "K76.5".to_string(), "K76.6".to_string(), "K76.7".to_string()],
            },
            "diab" => map! {
                "9" => vec!["250.0".to_string(), "250.1".to_string(), "250.2".to_string(), "250.3".to_string(), "250.8".to_string(), "250.9".to_string()],
                "10" => vec!["E10.0".to_string(), "E10.1".to_string(), "E10.6".to_string(), "E10.8".to_string(),"E10.9".to_string(),"E11.0".to_string(),"E11.1".to_string(),"E11.6".to_string(),"E11.8".to_string(),"E11.9".to_string(),"E13.0".to_string(),"E13.1".to_string(),"E13.6".to_string(),"E13.8".to_string(),"E13.9".to_string()],
            },
            "dia_w_c" => map! {
                "9" => vec!["250.4".to_string(), "250.5".to_string(), "250.6".to_string(), "250.7".to_string()],
                "10" => vec!["E10.2".to_string(), "E10.3".to_string(), "E10.4".to_string(), "E10.5".to_string(), "E10.7".to_string(), "E11.2".to_string(), "E11.3".to_string(), "E11.4".to_string(), "E11.5".to_string(), "E11.7".to_string(), "E13.2".to_string(), "E13.3".to_string(), "E13.4".to_string(), "E13.5".to_string(), "E13.7".to_string()],
            },
            "hp" => map! {
                "9" => vec!["334.1".to_string(), "342".to_string(), "343".to_string(), "344.0".to_string(),"344.1".to_string(),"344.2".to_string(),"344.3".to_string(),"344.4".to_string(),"344.5".to_string(),"344.6".to_string(),"344.9".to_string()],
                "10" => vec!["G04.1".to_string(), "G11.4".to_string(), "G80.1".to_string(), "G80.2".to_string(), "G81".to_string(), "G82".to_string(), "G83.0".to_string(),"G83.1".to_string(),"G83.2".to_string(),"G83.3".to_string(),"G83.4".to_string(),"G83.9".to_string()],
            },
            "mrend" => map! {
                "9" => vec!["403.00".to_string(),"403.10".to_string(), "403.90".to_string(), "404.00".to_string(), "404.01".to_string(), "404.10".to_string(), "404.11".to_string(), "404.90".to_string(), "404.91".to_string(), "584".to_string(), "585.6".to_string(), "589".to_string()],
                "10" => vec!["I12.9".to_string(),"I13.0".to_string(),"I13.10".to_string(),"N03".to_string(), "N05".to_string(),"N18.1".to_string(), "N18.2".to_string(),"N18.3".to_string(),"N18.4".to_string(),"N18.9".to_string(), "Z49.0".to_string()],
            },
            "srend" => map! {
                "9" => vec!["403.01".to_string(), "403.11".to_string(), "403.91".to_string(), "404.02".to_string(), "404.03".to_string(), "404.12".to_string(), "404.13".to_string(), "404.92".to_string(), "404.93".to_string(),"582".to_string(),"583.0".to_string(),"583.1".to_string(),"583.2".to_string(),"583.3".to_string(),"583.4".to_string(),"583.5".to_string(),"583.6".to_string(),"583.7".to_string(),"585.5".to_string(), "585.6".to_string(),"586".to_string(),"588.0".to_string(),"V42.0".to_string(),"V45.1".to_string(),"V56".to_string()],
                "10" => vec!["I12.0".to_string(), "I13.11".to_string(),"I13.2".to_string(),"N18.5".to_string(),"N18.6".to_string(),"N19".to_string(), "N25.0".to_string(),"Z49".to_string(),"Z94.0".to_string(), "Z99.2".to_string()],
            },
            "aids" => map! {
                "9" => vec!["112".to_string(), "180".to_string(), "114".to_string(), "117.5".to_string(), "007.4".to_string(), "078.5".to_string(), "348.3".to_string(), "054".to_string(), "115".to_string(), "007.2".to_string(), "176".to_string(), "200".to_string(), "201".to_string(), "202".to_string(), "203".to_string(), "204".to_string(), "205".to_string(), "206".to_string(), "207".to_string(), "208".to_string(), "209".to_string(), "031".to_string(), "010".to_string(), "011".to_string(), "012".to_string(), "013".to_string(), "014".to_string(), "015".to_string(), "016".to_string(), "017".to_string(), "018".to_string(), "136.3".to_string(), "V12.61".to_string(), "046.3".to_string(), "003.1".to_string(), "130".to_string(), "799.4".to_string()],
                "10" => vec!["B37".to_string(), "C53".to_string(), "B38".to_string(), "B45".to_string(), "A07.2".to_string(), "B25".to_string(), "G93.4".to_string(), "B00".to_string(), "B39".to_string(), "A07.3".to_string(), "C46".to_string(), "C81".to_string(), "C82".to_string(), "C83".to_string(), "C84".to_string(), "C85".to_string(), "C86".to_string(), "C87".to_string(), "C88".to_string(), "C89".to_string(), "C90".to_string(), "C91".to_string(), "C92".to_string(), "C93".to_string(), "C94".to_string(), "C95".to_string(), "C96".to_string(), "A31".to_string(), "A15".to_string(), "A16".to_string(), "A17".to_string(), "A18".to_string(), "A19".to_string(), "B59".to_string(), "Z87.01".to_string(), "A81.2".to_string(), "A02.1".to_string(), "B58".to_string(), "R64".to_string()],
            },
            "hiv" => map! {
                "9" => vec!["042".to_string()],
                "10" => vec!["B20".to_string()],
            },
            "mst" => map! {
                "9" => vec!["196".to_string(), "197".to_string(), "198".to_string(), "199.0".to_string()],
                "10" => vec!["C77".to_string(), "C78".to_string(), "C79".to_string(), "C80.0".to_string(), "C80.2".to_string()],
            },
            "mal" => map! {
                "9" => vec!["14".to_string(), "15".to_string(), "16".to_string(), "170".to_string(), "171".to_string(), "172".to_string(), "174".to_string(), "175".to_string(), "176".to_string(), "179".to_string(), "18".to_string(), "190".to_string(), "191".to_string(), "192".to_string(), "193".to_string(), "194".to_string(), "195".to_string(), "199.1".to_string(), "200".to_string(), "201".to_string(), "202".to_string(), "203".to_string(), "204".to_string(), "205".to_string(), "206".to_string(), "207".to_string(), "208".to_string(), "238.6".to_string()],
                "10" => vec!["C0".to_string(), "C1".to_string(), "C2".to_string(), "C30".to_string(), "C31".to_string(), "C32".to_string(), "C33".to_string(), "C34".to_string(), "C37".to_string(), "C38".to_string(), "C39".to_string(), "C40".to_string(), "C41".to_string(), "C43".to_string(), "C45".to_string(), "C46".to_string(), "C47".to_string(), "C48".to_string(), "C49".to_string(), "C50".to_string(), "C51".to_string(),"C52".to_string(),"C53".to_string(),"C54".to_string(),"C55".to_string(),"C56".to_string(),"C57".to_string(), "C58".to_string(), "C60".to_string(),"C61".to_string(),"C62".to_string(), "C63".to_string(), "C76".to_string(), "C80.1".to_string(), "C81".to_string(), "C82".to_string(), "C83".to_string(), "C84".to_string(), "C85".to_string(), "C88".to_string(), "C9".to_string()],
            },
            "Obesity" => map! {
                "9" => vec!["278.0".to_string()],
                "10" => vec!["E66".to_string()],
            },
            "WL" => map! {
                "9" => vec!["260".to_string(),"261".to_string(),"262".to_string(),"263".to_string(),"783.2".to_string(),"799.4".to_string()],
                "10" => vec!["E40".to_string(),"E41".to_string(),"E42".to_string(),"E43".to_string(),"E44".to_string(),"E45".to_string(),"E46".to_string(),"R63.4".to_string(),"R64".to_string()],
            },
            "Alcohol" => map! {
                "9" => vec!["265.2".to_string(),"291.1".to_string(),"291.2".to_string(),"291.3".to_string(),"291.5".to_string(),"291.8".to_string(),"291.9".to_string(),"303.0".to_string(),"303.9".to_string(),"305.0".to_string(),"357.5".to_string(),"425.5".to_string(),"535.3".to_string(),"571.0".to_string(),"571.1".to_string(),"5712".to_string(),"5713".to_string(),"980".to_string(),"V113".to_string()],
                "10" => vec!["F10".to_string(),"E52".to_string(),"G62.1".to_string(),"I42.6".to_string(),"K29.2".to_string(),"K70.0".to_string(),"K70.3".to_string(),"K70.9".to_string(),"T51".to_string(),"Z50.2".to_string(),"Z71.4".to_string(),"Z72.1".to_string()],
            },
            "Drug" => map! {
                "9" => vec!["292".to_string(),"304".to_string(),"305.2".to_string(),"305.3".to_string(),"305.4".to_string(),"305.5".to_string(),"305.6".to_string(),"305.7".to_string(),"305.8".to_string(),"305.9".to_string(),"V65.42".to_string()],
                "10" => vec!["F11".to_string(),"F12".to_string(),"F13".to_string(),"F14".to_string(),"F15".to_string(),"F16".to_string(),"F18".to_string(),"F19".to_string(),"Z71.5".to_string(),"Z72.2".to_string()],
            },
            "Psycho" => map! {
                "9" => vec!["293.8".to_string(),"295".to_string(),"296.04".to_string(),"296.14".to_string(),"296.44".to_string(),"296.54".to_string(),"297".to_string(),"298".to_string()],
                "10" => vec!["F20".to_string(),"F22".to_string(),"F23".to_string(),"F24".to_string(),"F25".to_string(),"F28".to_string(),"F29".to_string(),"F30.2".to_string(),"F31.2".to_string(),"F31.5".to_string()],
            },
            "Dep" => map! {
                "9" => vec!["296.2".to_string(),"296.3".to_string(),"296.5".to_string(),"300.4".to_string(),"309".to_string(),"311".to_string()],
                "10" => vec!["F20.4".to_string(),"F31.3".to_string(),"F31.4".to_string(),"F31.5".to_string(),"F32".to_string(),"F33".to_string(),"F34.1".to_string(),"F41.2".to_string(),"F43.2".to_string()],
            },
        }
    });
