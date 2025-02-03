#include <cstdlib>
#include <cstring>
#include <map>
#include <mutex>

extern "C" {

#include "error_codes.h"
#include "identity.h"
#include "secret.h"

#ifdef PCD_CONFIG_SECRET_REQUESTER
#include "secret_requester.h"
#endif

#include "log.h"
}

extern "C" int pcd_request_secret(pcd_identity_t *data_owner_identity,
                                  const char *delegator_address,
                                  pcd_secret_t **output_secret) {
  return 0;
}

bool operator==(const pcd_identity_t &i1, const pcd_identity_t &i2) {
  int i;
  for (i = 0; i < 16; i++) {
    if (i1.value[i] != i2.value[i]) {
      return false;
    }
  }
  return true;
}

bool operator<(const pcd_identity_t &i1, const pcd_identity_t &i2) {
  int i;
  for (i = 0; i < 16; i++) {
    if (i1.value[i] < i2.value[i]) {
      return true;
    }
  }
  return false;
};

typedef struct {
  uint32_t counter;
  pcd_secret_t *secret;
} pcd_secret_store_entry_t;

static std::mutex pcd_secret_mutex;
static std::map<pcd_identity_t, pcd_secret_store_entry_t> pcd_secret_store;

extern "C" int pcd_secret_register(pcd_identity_t *id,
                                   pcd_secret_t *input_secret) {
  std::pair<std::map<pcd_identity_t, pcd_secret_store_entry_t>::iterator, bool>
      ret;

  pcd_secret_store_entry_t entry;

  pcd_secret_mutex.lock();

  // malloc only if non-exist
  entry.counter = 1;

  ret = pcd_secret_store.insert(
      std::pair<pcd_identity_t, pcd_secret_store_entry_t>(*id, entry));
  if (ret.second == false) {
    ret.first->second.counter += 1;
    pcd_secret_mutex.unlock();
    pcd_log("INFO: ");
    pcd_print_id(id);
    pcd_log(" secret already exists\n");
    return PCD_REINIT;
  }

  ret.first->second.secret =
      (pcd_secret_t *)malloc(sizeof(pcd_secret_t) + input_secret->secret_size);
  // pcd_log("INFO: secret size is %d\n", input_secret->secret_size);
  if (ret.first->second.secret == NULL) {
    pcd_secret_mutex.unlock();
    pcd_log_error("ERROR: Failed to allocate secret\n");
    return PCD_MEMERR;
  }
  ret.first->second.secret->secret_size = input_secret->secret_size;
  memcpy(ret.first->second.secret->secret, input_secret->secret,
         input_secret->secret_size);

  pcd_secret_mutex.unlock();
  // pcd_log("INFO: ");
  // pcd_print_id(id);
  // pcd_log(" secret registered\n");
  return PCD_OK;
}

extern "C" int pcd_secret_release(pcd_identity_t *id) {
  std::map<pcd_identity_t, pcd_secret_store_entry_t>::iterator it;
  pcd_secret_t *secret;

  pcd_secret_mutex.lock();

  // map.find(*id) doesn't seem to work for no good reason...
  for (it = pcd_secret_store.begin(); it != pcd_secret_store.end(); it++) {
    if (it->first == *id) {
      break;
    }
  }
  if (it == pcd_secret_store.end()) {
    pcd_secret_mutex.unlock();

    pcd_log("INFO: ");
    pcd_print_id(id);
    pcd_log(" secret not found\n");
    return PCD_NOT_FOUND;
  }

  it->second.counter -= 1;

  // Remove the entry if no one else is using it
  if (it->second.counter <= 0) {
    secret = it->second.secret;
    pcd_secret_store.erase(it);

    // Free after erase to make sure no race can happen
    free(secret);
  }

  pcd_secret_mutex.unlock();
  return PCD_OK;
}

extern "C" int pcd_secret_retrieve(pcd_identity_t *id,
                                   pcd_secret_t **output_secret) {
  std::map<pcd_identity_t, pcd_secret_store_entry_t>::iterator it;

  pcd_secret_mutex.lock();

  // map.find(*id) doesn't seem to work for no good reason...
  for (it = pcd_secret_store.begin(); it != pcd_secret_store.end(); it++) {
    if (it->first == *id) {
      break;
    }
  }

  if (it == pcd_secret_store.end()) {
    pcd_secret_mutex.unlock();
    // pcd_log("INFO: ");
    // pcd_print_id(id);
    // pcd_log(" secret not found\n");
    // pcd_log("INFO: secret store has keys for\n");
    /*for (it = pcd_secret_store.begin(); it != pcd_secret_store.end(); it++) {
            pcd_log("      + ");
            pcd_print_id((pcd_identity_t *)(&it->first));
            pcd_log("\n");
            if (it->first == *id) {
                    pcd_log("INFO: WTF? You are finding one!\n");
            }
    }*/
    return PCD_NOT_FOUND;
  }

  // pcd_log("INFO: ");
  // pcd_print_id(id);
  // pcd_log(" secret found\n");

  it->second.counter += 1;
  *output_secret = it->second.secret;
  pcd_secret_mutex.unlock();

  return PCD_OK;
}

// #ifdef PCD_CONFIG_SECRET_REQUESTER

static std::mutex pcd_secret_fetch_lock;

extern "C" int pcd_secret_fetch(pcd_identity_t *id,
                                pcd_delegator_addr_t *delegator_addr,
                                pcd_secret_t **output_secret) {
  std::map<pcd_identity_t, pcd_secret_store_entry_t>::iterator it;
  int status;

  if (id == NULL || delegator_addr == NULL || output_secret == NULL) {
    return PCD_NULL_ARG;
  }

  if (pcd_secret_retrieve(id, output_secret) == PCD_OK) {
    return PCD_OK;
  }

  // Only allow one thread to fetch secret to avoid race
  pcd_secret_fetch_lock.lock();

  // Not found in the
  status = pcd_request_secret(id, (char *)delegator_addr, output_secret);
  if (status != PCD_OK) {
    pcd_secret_fetch_lock.unlock();
    return status;
  }

  status = pcd_secret_register(id, *output_secret);

  pcd_secret_fetch_lock.unlock();

  free(*output_secret);
  pcd_secret_retrieve(id, output_secret);
  // pcd_secret_release(id);

  return status;
}

// #endif
