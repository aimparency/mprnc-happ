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
    error::{
        ZomeApiResult,
        ZomeApiError
    },
	prelude::{
        EntryType,
		LinkMatch,
	}
};
use hdk::holochain_core_types::{
    entry::Entry,
    dna::entry_types::Sharing,
};

use hdk::holochain_persistence_api::{
    cas::content::Address,
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
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Connection {
    contributing: Address, 
    receiving: Address,
    contribution: u32
}

pub fn handle_create_aim(
	title: String, 
	description: String, 
    effort: Effort, 
	timestamp_ms: i64,
    color: [char; 6], 
    tags: Vec<String>,
) -> ZomeApiResult<Address> {
	let aim = Aim {
		title,
		description, 
        color, 
        effort, 
		timestamp_ms, 
        tags,
	};
    let entry = Entry::App("aim".into(), aim.into());
    let address = hdk::commit_entry(&entry)?;
	hdk::link_entries(
		&hdk::AGENT_ADDRESS.clone(), 
		&address, 
		"created_aim",
		""
	)?;
    Ok(address)
}

pub fn handle_update_aim(
    aim_address: Address, 
	title: String, 
	description: String, 
    effort: Effort, 
	timestamp_ms: i64,
    color: [char; 6], 
    tags: Vec<String>, 
) -> ZomeApiResult<Address>{
	let aim = Aim {
		title,
		description, 
        color, 
        effort, 
		timestamp_ms, 
        tags,
	};
    let entry = Entry::App("aim".into(), aim.into());
    hdk::update_entry(entry, &aim_address)
}

pub fn handle_create_receiving_aim(
	title: String, 
	description: String, 
    effort: Effort, 
	timestamp_ms: i64,
    color: [char; 6], 
    tags: Vec<String>, 
    contributing_aim_address: Address, 
) -> ZomeApiResult<Address> {
    let new_aim_address = handle_create_aim(title, description, effort, timestamp_ms, color, tags)?;
    handle_create_connection(contributing_aim_address, new_aim_address.clone(), 1)?;
    Ok(new_aim_address)
}

pub fn handle_create_contributing_aim(
	title: String, 
	description: String, 
    effort: Effort, 
	timestamp_ms: i64,
    color: [char; 6], 
    tags: Vec<String>, 
    receiving_aim_address: Address
) -> ZomeApiResult<Address> {
    let new_aim_address = handle_create_aim(title, description, effort, timestamp_ms, color, tags)?; 
    handle_create_connection(new_aim_address.clone(), receiving_aim_address, 1)?; 
    Ok(new_aim_address)
}

pub fn handle_create_connection(
    contributing_aim_address: Address, 
    receiving_aim_address: Address, 
    contribution: u32,
) -> ZomeApiResult<()> {
    let connection = Connection {
        contributing: contributing_aim_address.clone(), 
        receiving: receiving_aim_address.clone(), 
        contribution
    }; 
    let entry = Entry::App("connection".into(), connection.into()); 
    let connection_address = hdk::commit_entry(&entry)?;

    hdk::link_entries(&contributing_aim_address, &connection_address, "contributes_to_connection", "")?;
    hdk::link_entries(&connection_address, &receiving_aim_address, "contributes_to_aim", "")?;
    hdk::link_entries(&receiving_aim_address, &connection_address, "receives_from_connection", "")?;
    hdk::link_entries(&connection_address, &contributing_aim_address, "receives_from_aim", "")?;

    Ok(())
}

pub fn handle_get_aims() -> ZomeApiResult<Vec<Aim>> {
	hdk::utils::get_links_and_load_type(
		&hdk::AGENT_ADDRESS, 
		LinkMatch::Exactly("created_aim"),
		LinkMatch::Any
	)
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct AimDetails {
    aim: Aim,
    // more stuff like roles in the future i hope
}

pub fn handle_get_aim_details(aim_address: Address) -> ZomeApiResult<AimDetails> {
    match hdk::get_entry(&aim_address) {
        Ok(option) => match option {
            Some(entry) => match entry {
                Entry::App(_, json_string) => match Aim::try_from(json_string.to_owned()) {
                    Ok(aim) => Ok(AimDetails {
                        aim
                    }),
                    Err(_) => Err(ZomeApiError::Internal("could not parse entry json string".into()))
                }
                _ => Err(ZomeApiError::Internal("not an app entry".into()))
            }, 
            None => Err(ZomeApiError::Internal("could not find this entry".into()))
        }, 
        Err(err) => Err(err)
    }
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct ConnectedAim{
    aim: Aim, 
    aim_address: Address, 
    connection: Connection,
    connection_address: Address, 
}

pub fn handle_get_agent_address() -> ZomeApiResult<Address> {
	Ok(hdk::AGENT_ADDRESS.clone())
}

pub fn handle_create_root_aim() -> ZomeApiResult<Address> {
    let aim = Aim {
        title: String::from("root aim"), 
        description: String::from("this is the single root aim of this agent. Some algorithms will use these root aims as the main source of collective will: using these aims as the only entrance of value flow before calculating the importance of all goals"),
        effort: Effort::Years(100), 
        timestamp_ms: 1594443995818, 
        color: ['5'; 6],
        tags: Vec::<String>::new(),
    };
    let entry = Entry::App("aim".into(), aim.into());
    let address = hdk::commit_entry(&entry)?;
    hdk::link_entries(
        &hdk::AGENT_ADDRESS.clone(), 
        &address, 
        "has_root_aim", 
        ""
    )
}

pub fn handle_get_root_aim_address_or_create() -> ZomeApiResult<Address> {
    match handle_get_root_aim_address(){
        Ok(option) => match option {
            Some(address) => Ok(address), 
            None => match handle_create_root_aim() {
                Ok(address) => Ok(address), 
                Err(err) => Err(err)
            }
        },
        Err(err) => Err(err) 
    }
}

pub fn handle_get_root_aim_address() -> ZomeApiResult<Option<Address>>{
    match hdk::get_links(
        &hdk::AGENT_ADDRESS.clone(), 
        LinkMatch::Exactly("has_root_aim"), 
        LinkMatch::Any
    ) {
        Ok(result) => Ok(match result.addresses().first() {
            Some(address) => Some(address.clone()), 
            None => None
        }),
        Err(err) => Err(err)
    }
}

pub fn handle_get_receiving_aims(
    contributing_aim_address: Address, 
) -> ZomeApiResult<Vec<ConnectedAim>> {
    get_connected_aims(contributing_aim_address, "contributes_to".into())
}

pub fn handle_get_contributing_aims(
    receiving_aim_address: Address, 
) -> ZomeApiResult<Vec<ConnectedAim>> {
    get_connected_aims(receiving_aim_address, "receives_from".into())
}

pub fn get_connected_aims (
    receiving_aim_address: Address, 
    relation: String
) -> ZomeApiResult<Vec<ConnectedAim>> {
    Ok( hdk::get_links(
        &receiving_aim_address, 
        LinkMatch::Exactly(&format!("{}_connection", relation)), 
        LinkMatch::Any
    )?.addresses().iter()
        .filter_map(|connection_address| match hdk::api::get_entry(connection_address) {
            Ok(connection_option) => match connection_option {
                Some(connection_entry) => match match hdk::get_links(
                        connection_address,
                        LinkMatch::Exactly(&format!("{}_aim", relation)),
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
				EntryType::AgentId, 
				link_type: "has_root_aim", 
				validation_package:  || {
					hdk::ValidationPackageDefinition::Entry
				},
				validation: | _validation_data: hdk::LinkValidationData | {
					Ok(())
				}
			),
			from!(
				EntryType::AgentId, 
				link_type: "created_aim", 
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
				link_type: "contributes_to_connection", 
				validation_package:  || {
					hdk::ValidationPackageDefinition::Entry
				},
				validation: | _validation_data: hdk::LinkValidationData | {
					Ok(())
				}
			), 
			from!(
				"aim", 
				link_type: "receives_from_connection", 
				validation_package:  || {
					hdk::ValidationPackageDefinition::Entry
				},
				validation: | _validation_data: hdk::LinkValidationData | {
					Ok(())
				}
			), 
			to!(
				"aim", 
				link_type: "contributes_to_aim", 
				validation_package:  || {
					hdk::ValidationPackageDefinition::Entry
				},
				validation: | _validation_data: hdk::LinkValidationData | {
					Ok(())
				}
			), 
			to!(
				"aim", 
				link_type: "receives_from_aim", 
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

    init: || { 
        match handle_create_root_aim() {
            Ok(_) => Ok(()),
            Err(error) => Err(format!("{}", error))
        }
    }

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {
        Ok(())
    }

    functions: [
		get_agent_address: {
			inputs: | |, 
			outputs: |address: ZomeApiResult<Address>|, 
			handler: handle_get_agent_address
		}
        create_root_aim: {
            inputs: | |,
            outputs: |address: ZomeApiResult<Address>|,
            handler: handle_create_root_aim 
        }
        get_root_aim_address_or_create: {
            inputs: | |,
            outputs: |address: ZomeApiResult<Address>|,
            handler: handle_get_root_aim_address_or_create
        }
        get_root_aim_address: {
            inputs: | |,
            outputs: |address: ZomeApiResult<Option<Address>>|,
            handler: handle_get_root_aim_address
        }
        create_aim: {
            inputs: |
                title:String, 
                description:String, 
                effort: Effort, 
                timestamp_ms: i64, 
                color: [char; 6],
                tags: Vec<String>
            |,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_create_aim
        }
        create_receiving_aim: {
            inputs: |
                title:String, 
                description:String, 
                effort: Effort, 
                timestamp_ms: i64, 
                color: [char; 6],
                tags: Vec<String>, 
                connected_aim_address: Address
            |,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_create_receiving_aim
        }
        create_contributing_aim: {
            inputs: |
                title:String, 
                description:String, 
                effort: Effort, 
                timestamp_ms: i64, 
                color: [char; 6],
                tags: Vec<String>, 
                connected_aim_address: Address
            |,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_create_contributing_aim
        }
        update_aim: {
            inputs: |
                aim_address: Address,
                title:String, 
                description:String, 
                effort: Effort, 
                timestamp_ms: i64, 
                color: [char; 6],
                tags: Vec<String>
            |,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_update_aim
        }
        get_aims: {
            inputs: | |,
            outputs: |result: ZomeApiResult<Vec<Aim>>|,
            handler: handle_get_aims 
        }
        get_aim_details: {
            inputs: | aim_address: Address |,
            outputs: | result: ZomeApiResult<AimDetails> |, 
            handler: handle_get_aim_details 
        }
        create_connection: {
            inputs: |contributing_aim_address: Address, receiving_aim_address: Address, contribution: u32 |,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_create_connection 
        }
        get_contributing_aims: {
            inputs: |aim_address: Address|,
            outputs: |result: ZomeApiResult<Vec<ConnectedAim>>|, 
            handler: handle_get_contributing_aims
        }
        get_receiving_aims: {
            inputs: |aim_address: Address|,
            outputs: |result: ZomeApiResult<Vec<ConnectedAim>>|, 
            handler: handle_get_receiving_aims
        }
    ]

    traits: {
        hc_public [
			create_aim, 
			get_aims,
            get_aim_details,
            get_agent_address,
            get_root_aim_address, 
            get_root_aim_address_or_create, 
            create_root_aim, 
            create_receiving_aim,
            create_contributing_aim,
            update_aim,
            create_connection, 
            get_contributing_aims,
            get_receiving_aims
		]
    }
}
