#[macro_use]
extern crate hdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate holochain_json_derive;

use hdk::{
    error::ZomeApiResult,
	prelude::{
		LinkMatch, 
		EntryType
	},
	holochain_persistence_api::{
		cas::content::Address,
        hash::HashString,
	},
	holochain_core_types::{
		entry::{
            Entry,
        },
		dna::entry_types::Sharing,
	},
	holochain_json_api::{
		error::JsonError,
		json::JsonString,
	}
};

use std::convert::TryFrom; 

// see https://developer.holochain.org/api/0.0.47-alpha1/hdk/ for info on using the hdk library

// This is a sample zome that defines an entry type "MyEntry" that can be committed to the
// agent's chain via the exposed function create_my_entry

#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
pub struct Profile{
    name: String,
	creator: Address
}

#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
pub struct AddressAndEntry{ //TODO make this generic, not only for Profile
    entry: Profile, 
    address: HashString
}


pub fn handle_get_my_agent_address() -> ZomeApiResult<Address> {
	Ok(hdk::AGENT_ADDRESS.clone())
}

pub fn handle_create_profile(name: String) -> ZomeApiResult<Address> {
	let profile = Profile {
		name: name.clone(), 
		creator: hdk::AGENT_ADDRESS.clone()
	};
    let entry = Entry::App("profile".into(), profile.into());
    let address = hdk::commit_entry(&entry)?;
	hdk::link_entries(
		&hdk::AGENT_ADDRESS.clone(), 
		&address,
		"agent_created_profile",
		""
	)?;
    Ok(address)
}

pub fn handle_get_my_profiles() -> ZomeApiResult<Vec<AddressAndEntry>> {
    Ok( hdk::get_links(
		&hdk::AGENT_ADDRESS.clone(), 
		LinkMatch::Exactly("agent_created_profile"), 
		LinkMatch::Any
	)?.addresses().iter()
        .filter_map::<AddressAndEntry,_>(|address| {
            match hdk::api::get_entry(&address) {
                Ok(option) => {
                    return match option {
                        Some(entry) => {
                            match entry {
                                Entry::App(_, entry_value) => {
                                    return match Profile::try_from(entry_value.to_owned()) {
                                        Ok(profile) => Some(
                                            AddressAndEntry {
                                                address: address.clone(),
                                                entry: profile
                                            }
                                        ), 
                                        Err(_) => None
                                    }
                                },
                                _ => None,
                            }
                        },
                        None => None,
                    }
                }, 
                Err(_) => None,
            }
        })
        .collect::<Vec<AddressAndEntry>>()
    )
}

pub fn handle_get_my_profiles_without_addresses() -> ZomeApiResult<Vec<Profile>> {
    hdk::utils::get_links_and_load_type(
        &hdk::AGENT_ADDRESS.clone(), 
        LinkMatch::Exactly("agent_created_profile"),
        LinkMatch::Any
    )
}

define_zome! {
    entries: [
		entry!(
			name: "profile",
			description: "one agent can create and manage multiple profiles",
			sharing: Sharing::Public,
			validation_package: || {
				hdk::ValidationPackageDefinition::Entry
			},
			validation: | _validation_data: hdk::EntryValidationData<Profile>| {
				Ok(())
			},
			links: [
				from!(
					EntryType::AgentId, 
					link_type: "agent_created_profile", 
					validation_package: || {
						hdk::ValidationPackageDefinition::Entry
					}, 
					validation: |_validation_data: hdk::LinkValidationData| {
						Ok(())
					}
				)
			]
		)
    ]

    init: || { Ok(()) }

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {
        Ok(())
    }

    functions: [
		get_my_agent_address: {
			inputs: | |, 
			outputs: |address: ZomeApiResult<Address>|, 
			handler: handle_get_my_agent_address
		}
        create_profile: {
            inputs: |name: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_create_profile 
        }
		get_my_profiles: {
			inputs: | |, 
			outputs: |result: ZomeApiResult<Vec<AddressAndEntry>>|, 
			handler: handle_get_my_profiles
		}
    ]

    traits: {
        hc_public [
			get_my_agent_address, 
			create_profile,
			get_my_profiles
		]
    }
}

