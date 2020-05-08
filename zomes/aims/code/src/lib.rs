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


#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
pub struct Aim {
    title: String,
	description: String, 
	timestamp_ms: i64,
}

pub fn handle_create_aim(
	title: String, 
	description: String, 
	profile: HashString, 
	timestamp_ms: i64
) -> ZomeApiResult<Address> {
	let aim = Aim {
		title,
		description, 
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
            inputs: |title:String, description:String, profile:HashString, timestamp_ms: i64|,
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
