// //! A simple policy engine.

// use policy_styx_sys::crypto::PcdSha256;
// use policy_styx_sys::data::PcdPayload;
// use policy_styx_sys::identity::PcdIdentity;

// pub type PolicyResult<T> = anyhow::Result<T>;

// const PCD_DEMO_POLICY_TYPE_PROGRAM_HASH: u8 = 0x01;
// const PCD_DEMO_POLICY_TYPE_CUSTODIAN: u8 = 0x02;

// /// A demo policy for styx.
// ///
// /// This policy does not care about how will the input policy be used.
// #[repr(C, packed)]
// #[derive(Default)]
// pub struct PcdDemoPolicyRule {
//     pub rule_type: u8,
//     pub data_custodian_id: PcdIdentity,
// }

// /// Evaluates the input payloads.
// ///
// /// # Arguments
// ///
// /// * `payload_ptr_array` - A pointer to an array of pointers to `PcdPayload`.
// /// * `payload_amount` - The number of payloads in the array.
// ///
// /// # Returns
// ///
// /// * `0` if the evaluation is successful.
// /// * `-1` if the evaluation fails.
// #[no_mangle]
// pub unsafe extern "C" fn eval_input(
//     payload_ptr_array: *const *const PcdPayload,
//     payload_amount: usize,
//     program_hash: *const u8,
//     program_hash_size: usize,
// ) -> i32 {
//     let program_hash: PcdSha256 =
//         match std::slice::from_raw_parts(program_hash, program_hash_size).try_into() {
//             Ok(hash) => hash,
//             Err(_) => return -1,
//         };

//     let policies = std::slice::from_raw_parts(payload_ptr_array, payload_amount)
//         .into_iter()
//         .map(|s| &**s)
//         .collect::<Vec<_>>();

//     // We next check the policy.
//     // If the policy is not satisfied, we return -1.
//     for p in policies {
//         // interpret the policy payload.
//         let rule = &*(p.payload.add(p.data_size as usize) as *const PcdDemoPolicyRule);

//         match rule.rule_type {
//             PCD_DEMO_POLICY_TYPE_PROGRAM_HASH => {
//                 // Check the program hash.
//                 if program_hash != [0u8; 32] {
//                     return -1;
//                 }
//             },
//             _ => return -2, // not implemented :(
//         }
//     }

//     0
// }

// /// Evaluates the output payloads.
// ///
// /// # Arguments
// ///
// /// * `payload_ptr_array` - A pointer to an array of pointers to `PcdPayload`.
// /// * `payload_amount` - The number of payloads in the array.
// /// * `data` - A pointer to the data.
// /// * `data_size` - The size of the data.
// /// * `data_owner_id` - A pointer to the `PcdIdentity` of the data owner.
// /// * `attributes` - A pointer to the attributes.
// /// * `attribute_size` - The size of the attributes.
// ///
// /// # Returns
// ///
// /// * `0` if the evaluation is successful.
// /// * `-1` if the evaluation fails.
// #[no_mangle]
// pub unsafe extern "C" fn eval_output(
//     payload_ptr_array: *const *const PcdPayload,
//     payload_amount: usize,
//     data_owner_id: *const PcdIdentity,
// ) -> i32 {
//     let policies = std::slice::from_raw_parts(payload_ptr_array, payload_amount)
//         .into_iter()
//         .map(|s| &**s)
//         .collect::<Vec<_>>();

//     // We next check the policy.
//     // If the policy is not satisfied, we return -1.
//     for p in policies {
//         // interpret the policy payload.
//         let rule = &*(p.payload.add(p.data_size as usize) as *const PcdDemoPolicyRule);

//         match rule.rule_type {
//             PCD_DEMO_POLICY_TYPE_PROGRAM_HASH => continue, // we do not care.
//             PCD_DEMO_POLICY_TYPE_CUSTODIAN => {
//                 if *data_owner_id != rule.data_custodian_id {
//                     println!("data owner id does not match");
//                     return -1;
//                 }
//             }, // not implemented :(
//             _ => return -3,                                // not implemented :(
//         }
//     }

//     0
// }
