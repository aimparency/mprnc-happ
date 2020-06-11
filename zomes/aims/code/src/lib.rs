#[macro_use]
extern crate hdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate holochain_json_derive;

use hdk::{
    entry_definition::ValidatingEntryType,
    error::ZomeApiResult,
	prelude::{
		LinkMatch,
	}
};
use hdk::holochain_core_types::{
    entry::Entry,
    dna::entry_types::Sharing,
};

use hdk::holochain_persistence_api::{
    cas::content::Address,
	hash::HashString,
};

use hdk::holochain_json_api::{
    error::JsonError,
    json::JsonString,
};


#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub enum Effort {
    Minutes(u64), 
    Hours(u64), 
    Days(u64), 
    Weeks(u64), 
    Months(u64), 
    Years(u64),
}

impl ToString for Effort {
    fn to_string(&self) -> String {
        match self {
            Effort::Minutes(m) => m.to_string() + "min",
            Effort::Hours(h) => h.to_string() + "h", 
            Effort::Days(d) => d.to_string() + "d", 
            Effort::Weeks(w) => w.to_string() + "w", 
            Effort::Months(m) => m.to_string() + "m",
            Effort::Years(y) => y.to_string() + "y"
        }
    }
}

impl Effort {
    fn from_string(mut s: String) -> Effort {
        s.retain(|c| !c.is_whitespace()); 
        let len = s.chars().count();
        let cutoff = | n | {
            let number_str: String = s.chars().take(len - n).collect();
            number_str.parse::<u64>().unwrap()
        };
        match s.chars().last().unwrap() {
            'n' => Effort::Minutes(cutoff(3)), 
            'h' => Effort::Hours(cutoff(1)), 
            'd' => Effort::Days(cutoff(1)), 
            'w' => Effort::Weeks(cutoff(1)), 
            'm' => Effort::Months(cutoff(1)), 
            'y' => Effort::Years(cutoff(1)),
            _ => Effort::Days(1u64) // default
        }
    }
}

#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
pub struct Aim {
    title: String,
	description: String, 
    effort: Effort, 
	timestamp_ms: i64,
}

pub fn handle_create_aim(
	title: String, 
	description: String, 
    effort_str: String, 
	profile: HashString, 
	timestamp_ms: i64
) -> ZomeApiResult<Address> {
	let aim = Aim {
		title,
		description, 
        effort: Effort::from_string(effort_str), 
		timestamp_ms, 
	};
    let entry = Entry::App("aim".into(), aim.into());
    let address = hdk::commit_entry(&entry)?;
	hdk::link_entries(
		&profile, 
		&address, 
		"profile_created_aim",
		""
	)?;
    Ok(address)
}

pub fn handle_get_aims(creator: Address) -> ZomeApiResult<Vec<Aim>> {
	hdk::utils::get_links_and_load_type(
		&creator, 
		LinkMatch::Exactly("profile_created_aim"),
		LinkMatch::Any
	)
}

fn definition() -> ValidatingEntryType {
    entry!(
        name: "aim",
        description: "this is an aim of some agent",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },

        validation: | _validation_data: hdk::EntryValidationData<Aim>| {
            Ok(())
        },
		links: [
			from!(
				"profile", 
				link_type: "profile_created_aim", 
				validation_package:  || {
					hdk::ValidationPackageDefinition::Entry
				},
				validation: | _validation_data: hdk::LinkValidationData | {
					Ok(())
				}
			)
		]
    )
}

define_zome! {
    entries: [
       definition()
    ]

    init: || { Ok(()) }

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {
        Ok(())
    }

    functions: [
        create_aim: {
            inputs: |title:String, description:String, effort_str: String, profile:HashString, timestamp_ms: i64|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_create_aim
        }
        get_aims: {
            inputs: |profile: Address|,
            outputs: |result: ZomeApiResult<Vec<Aim>>|,
            handler: handle_get_aims 
        }
    ]

    traits: {
        hc_public [
			create_aim, 
			get_aims
		]
    }
}
