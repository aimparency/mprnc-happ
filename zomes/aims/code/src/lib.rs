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

use std::convert::TryFrom;


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
/*    fn from_string(mut s: String) -> Effort {
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
    }*/
}

#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
pub struct Aim {
    title: String,
	description: String, 
    effort: Effort, 
	timestamp_ms: i64,
    color: [char; 6],
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Connection {
    contributing: Address, 
    benefited: Address,
    contribution: u32
}

pub fn handle_create_aim(
	title: String, 
	description: String, 
    effort: Effort, 
	profile: HashString, 
	timestamp_ms: i64,
    color: [char; 6]
) -> ZomeApiResult<Address> {
	let aim = Aim {
		title,
		description, 
        color, 
        effort, 
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

pub fn handle_create_connection(
    contributing_aim_address: Address, 
    benefited_aim_address: Address, 
    contribution: u32,
) -> ZomeApiResult<()> {
    let connection = Connection {
        contributing: contributing_aim_address.clone(), 
        benefited: benefited_aim_address.clone(), 
        contribution
    }; 
    let entry = Entry::App("connection".into(), connection.into()); 
    let connection_address = hdk::commit_entry(&entry)?;

    hdk::link_entries(&contributing_aim_address, &connection_address, "contributes_to", "")?;
    hdk::link_entries(&connection_address, &benefited_aim_address, "contributes_to", "")?;
    hdk::link_entries(&benefited_aim_address, &connection_address, "benefits_from", "")?;
    hdk::link_entries(&connection_address, &benefited_aim_address, "benefits_from", "")?;

    Ok(())
}

pub fn handle_get_aims(creator: Address) -> ZomeApiResult<Vec<Aim>> {
	hdk::utils::get_links_and_load_type(
		&creator, 
		LinkMatch::Exactly("profile_created_aim"),
		LinkMatch::Any
	)
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct ConnectedAim{
    aim: Aim, 
    aim_address: Address, 
    connection: Connection, 
    connection_address: Address, 
}

pub fn handle_get_benefited_aims(
    contributing_aim_address: Address, 
) -> ZomeApiResult<Vec<ConnectedAim>> {
    get_connected_aims(contributing_aim_address, "contributes_to")
}

pub fn handle_get_contributing_aims(
    benefited_aim_address: Address, 
) -> ZomeApiResult<Vec<ConnectedAim>> {
    get_connected_aims(benefited_aim_address, "benefited_from")
}

pub fn get_connected_aims (
    benefited_aim_address: Address, 
    relation: &str
) -> ZomeApiResult<Vec<ConnectedAim>> {
    Ok( hdk::get_links(
        &benefited_aim_address, 
        LinkMatch::Exactly(relation), 
        LinkMatch::Any
    )?.addresses().iter()
        .filter_map(|connection_address| match hdk::api::get_entry(connection_address) {
            Ok(connection_option) => match connection_option {
                Some(connection_entry) => match match hdk::get_links(
                        connection_address,
                        LinkMatch::Exactly(relation),
                        LinkMatch::Any
                    ) {
                    Ok(result) => result.addresses().iter()
                        .map(|aim_address| match hdk::api::get_entry(aim_address) {
                            Ok(aim_option) => match aim_option {
                                Some(aim_entry) => match connection_entry.clone(){
                                    Entry::App(_, connection_value) => match aim_entry {
                                        Entry::App(_, aim_value) => 
                                            match Connection::try_from(connection_value.to_owned()) {
                                                Ok(connection) => 
                                                    match Aim::try_from(aim_value.to_owned()) {
                                                        Ok(aim) => Some(ConnectedAim {
                                                            aim, 
                                                            aim_address: aim_address.clone().into(), 
                                                            connection, 
                                                            connection_address: connection_address.clone().into()
                                                        }), 
                                                        Err(_) => None
                                                    }
                                                Err(_) => None
                                            },
                                        
                                        _ => None
                                    }, 
                                    _ => None
                                }, 
                                None => None
                            },
                            Err(_) => None
                        })
                        .next(),
                    Err(_) => None
                } {
                    Some(connected_aim) => connected_aim, 
                    None => None
                },
                None => None
            },
            Err(_) => None
        })
        .collect::<Vec<ConnectedAim>>() 
    )
}

fn aim_entry_definition() -> ValidatingEntryType {
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

fn connection_entry_definition() -> ValidatingEntryType {
    entry!(
        name: "connection", 
        description: "this is a connection between aims which expresses contribution of one aim to another", 
        sharing: Sharing::Public,  
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: | _validation_data: hdk::EntryValidationData<Connection>| {
            Ok(())
        },
		links: [
			from!(
				"aim", 
				link_type: "contributes_to", 
				validation_package:  || {
					hdk::ValidationPackageDefinition::Entry
				},
				validation: | _validation_data: hdk::LinkValidationData | {
					Ok(())
				}
			), 
			from!(
				"aim", 
				link_type: "benefits_from", 
				validation_package:  || {
					hdk::ValidationPackageDefinition::Entry
				},
				validation: | _validation_data: hdk::LinkValidationData | {
					Ok(())
				}
			), 
			to!(
				"aim", 
				link_type: "contributes_to", 
				validation_package:  || {
					hdk::ValidationPackageDefinition::Entry
				},
				validation: | _validation_data: hdk::LinkValidationData | {
					Ok(())
				}
			), 
			to!(
				"aim", 
				link_type: "benefits_from", 
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
       aim_entry_definition(), 
       connection_entry_definition()
    ]

    init: || { Ok(()) }

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {
        Ok(())
    }

    functions: [
        create_aim: {
            inputs: |
                title:String, 
                description:String, 
                effort: Effort, 
                profile:HashString, 
                timestamp_ms: i64, 
                color: [char; 6]
            |,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_create_aim
        }
        get_aims: {
            inputs: |profile: Address|,
            outputs: |result: ZomeApiResult<Vec<Aim>>|,
            handler: handle_get_aims 
        }
        create_connection: {
            inputs: |contributing_aim_address: Address, benefited_aim_address: Address, contribution: u32 |,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_create_connection 
        }
        get_contributing_aims: {
            inputs: |benefited_aim_address: Address|,
            outputs: |result: ZomeApiResult<Vec<ConnectedAim>>|, 
            handler: handle_get_contributing_aims
        }
        get_benefited_aims: {
            inputs: |contributing_aim_address: Address|,
            outputs: |result: ZomeApiResult<Vec<ConnectedAim>>|, 
            handler: handle_get_benefited_aims
        }
    ]

    traits: {
        hc_public [
			create_aim, 
			get_aims
		]
    }
}
