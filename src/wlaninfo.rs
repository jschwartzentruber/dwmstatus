use libc::{c_int, c_void, if_nametoindex, ETH_ALEN};
use std::{ffi, mem, ptr, slice};

pub struct WLanInfo {
    pub essid: Option<String>,
    bssid: [u8; ETH_ALEN as usize],
    pub quality: Option<f64>,
    //signal: ?,
    //noise: ?,
    pub bitrate: Option<f64>,
    pub frequency: Option<f64>
}

#[link(name = "nl-genl-3")]
extern {
    fn genl_connect(sk: *mut nl_sock) -> c_int;
    fn genl_ctrl_resolve(sk: *mut nl_sock, name: *const i8) -> c_int;
    fn genlmsg_attrdata(gnlh: *const genlmsghdr, hdrlen: c_int) -> *mut nlattr;
    fn genlmsg_attrlen(gnlh: *const genlmsghdr, hdrlen: c_int) -> c_int;
    fn genlmsg_put(msg: *mut nl_msg, port: u32, seq: u32, family: c_int, hdrlen: c_int, flags: c_int, cmd: u8, version: u8) -> *mut c_void;
}

#[link(name = "nl-3")]
extern {
    fn nl_send_sync(sk: *mut nl_sock, msg: *mut nl_msg) -> c_int;
    fn nl_socket_alloc() -> *mut nl_sock;
    fn nl_socket_free(isk: *mut nl_sock);
    fn nl_socket_modify_cb(sk: *mut nl_sock, type_: nl_cb_type, kind: nl_cb_kind, func: extern fn(*mut nl_msg, arg: *mut WLanInfo) -> c_int, arg: *mut WLanInfo) -> c_int;
    fn nla_data(nla: *const nlattr) -> *mut c_void;
    fn nla_get_s32(nla: *mut nlattr) -> i32;
    fn nla_get_u16(nla: *mut nlattr) -> u16;
    fn nla_get_u32(nla: *mut nlattr) -> u32;
    fn nla_get_u8(nla: *mut nlattr) -> u8;
    fn nla_len(nla: *const nlattr) -> c_int;
    fn nla_parse(tb: *mut *mut nlattr, maxtype: c_int, head: *mut nlattr, len: c_int, policy: *mut nla_policy) -> c_int;
    fn nla_parse_nested(tb: *mut *mut nlattr, maxtype: c_int, nla: *mut nlattr, policy: *mut nla_policy) -> c_int;
    fn nla_put(msg: *mut nl_msg, attrtype: c_int, datalen: c_int, data: *const c_void) -> c_int;
    fn nla_put_u32(msg: *mut nl_msg, attrtype: c_int, value: u32) -> c_int;
    fn nlmsg_alloc() -> *mut nl_msg;
    fn nlmsg_data(nlh: *const nlmsghdr) -> *mut c_void;
    fn nlmsg_free(msg: *mut nl_msg);
    fn nlmsg_hdr(n: *mut nl_msg) -> *mut nlmsghdr;
}

#[repr(C)]
enum nla_types {
    NLA_UNSPEC,
    NLA_U8,
    NLA_U16,
    NLA_U32,
    NLA_U64,
    NLA_STRING,
    NLA_FLAG,
    NLA_MSECS,
    NLA_NESTED,
    NLA_NESTED_COMPAT,
    NLA_NUL_STRING,
    NLA_BINARY,
    NLA_S8,
    NLA_S16,
    NLA_S32,
    NLA_S64,
}

#[repr(C)]
enum nl_cb_action {
    NL_OK, // Proceed with wathever would come next
    NL_SKIP, // Skip this message
    NL_STOP, // Stop parsing altogether and discard remaining messages
}

#[repr(C)]
enum nl_cb_type {
    NL_CB_VALID, // Message is valid
    NL_CB_FINISH, // Last message in a series of multi part messages received
    NL_CB_OVERRUN, // Report received that data was lost
    NL_CB_SKIPPED, // Message wants to be skipped
    NL_CB_ACK, // Message is an acknowledge
    NL_CB_MSG_IN, // Called for every message received
    NL_CB_MSG_OUT, // Called for every message sent out except for nl_sendto()
    NL_CB_INVALID, // Message is malformed and invalid
    NL_CB_SEQ_CHECK, // Called instead of internal sequence number checking
    NL_CB_SEND_ACK, // Sending of an acknowledge message has been requested
    NL_CB_DUMP_INTR, // Flag NLM_F_DUMP_INTR is set in message
}

#[repr(C)]
enum nl_cb_kind {
    NL_CB_DEFAULT, // Default handlers (quiet)
    NL_CB_VERBOSE, // Verbose default handlers (error messages printed)
    NL_CB_DEBUG, // Debug handlers for debugging
    NL_CB_CUSTOM, // Customized handler specified by the user
}

#[repr(C)]
enum nl80211_attr {
    NL80211_ATTR_UNSPEC,
    NL80211_ATTR_WIPHY,
    NL80211_ATTR_WIPHY_NAME,
    NL80211_ATTR_IFINDEX,
    NL80211_ATTR_IFNAME,
    NL80211_ATTR_IFTYPE,
    NL80211_ATTR_MAC,
    NL80211_ATTR_KEY_DATA,
    NL80211_ATTR_KEY_IDX,
    NL80211_ATTR_KEY_CIPHER,
    NL80211_ATTR_KEY_SEQ,
    NL80211_ATTR_KEY_DEFAULT,
    NL80211_ATTR_BEACON_INTERVAL,
    NL80211_ATTR_DTIM_PERIOD,
    NL80211_ATTR_BEACON_HEAD,
    NL80211_ATTR_BEACON_TAIL,
    NL80211_ATTR_STA_AID,
    NL80211_ATTR_STA_FLAGS,
    NL80211_ATTR_STA_LISTEN_INTERVAL,
    NL80211_ATTR_STA_SUPPORTED_RATES,
    NL80211_ATTR_STA_VLAN,
    NL80211_ATTR_STA_INFO,
    NL80211_ATTR_WIPHY_BANDS,
    NL80211_ATTR_MNTR_FLAGS,
    NL80211_ATTR_MESH_ID,
    NL80211_ATTR_STA_PLINK_ACTION,
    NL80211_ATTR_MPATH_NEXT_HOP,
    NL80211_ATTR_MPATH_INFO,
    NL80211_ATTR_BSS_CTS_PROT,
    NL80211_ATTR_BSS_SHORT_PREAMBLE,
    NL80211_ATTR_BSS_SHORT_SLOT_TIME,
    NL80211_ATTR_HT_CAPABILITY,
    NL80211_ATTR_SUPPORTED_IFTYPES,
    NL80211_ATTR_REG_ALPHA2,
    NL80211_ATTR_REG_RULES,
    NL80211_ATTR_MESH_CONFIG,
    NL80211_ATTR_BSS_BASIC_RATES,
    NL80211_ATTR_WIPHY_TXQ_PARAMS,
    NL80211_ATTR_WIPHY_FREQ,
    NL80211_ATTR_WIPHY_CHANNEL_TYPE,
    NL80211_ATTR_KEY_DEFAULT_MGMT,
    NL80211_ATTR_MGMT_SUBTYPE,
    NL80211_ATTR_IE,
    NL80211_ATTR_MAX_NUM_SCAN_SSIDS,
    NL80211_ATTR_SCAN_FREQUENCIES,
    NL80211_ATTR_SCAN_SSIDS,
    NL80211_ATTR_GENERATION,
    NL80211_ATTR_BSS,
    NL80211_ATTR_REG_INITIATOR,
    NL80211_ATTR_REG_TYPE,
    NL80211_ATTR_SUPPORTED_COMMANDS,
    NL80211_ATTR_FRAME,
    NL80211_ATTR_SSID,
    NL80211_ATTR_AUTH_TYPE,
    NL80211_ATTR_REASON_CODE,
    NL80211_ATTR_KEY_TYPE,
    NL80211_ATTR_MAX_SCAN_IE_LEN,
    NL80211_ATTR_CIPHER_SUITES,
    NL80211_ATTR_FREQ_BEFORE,
    NL80211_ATTR_FREQ_AFTER,
    NL80211_ATTR_FREQ_FIXED,
    NL80211_ATTR_WIPHY_RETRY_SHORT,
    NL80211_ATTR_WIPHY_RETRY_LONG,
    NL80211_ATTR_WIPHY_FRAG_THRESHOLD,
    NL80211_ATTR_WIPHY_RTS_THRESHOLD,
    NL80211_ATTR_TIMED_OUT,
    NL80211_ATTR_USE_MFP,
    NL80211_ATTR_STA_FLAGS2,
    NL80211_ATTR_CONTROL_PORT,
    NL80211_ATTR_TESTDATA,
    NL80211_ATTR_PRIVACY,
    NL80211_ATTR_DISCONNECTED_BY_AP,
    NL80211_ATTR_STATUS_CODE,
    NL80211_ATTR_CIPHER_SUITES_PAIRWISE,
    NL80211_ATTR_CIPHER_SUITE_GROUP,
    NL80211_ATTR_WPA_VERSIONS,
    NL80211_ATTR_AKM_SUITES,
    NL80211_ATTR_REQ_IE,
    NL80211_ATTR_RESP_IE,
    NL80211_ATTR_PREV_BSSID,
    NL80211_ATTR_KEY,
    NL80211_ATTR_KEYS,
    NL80211_ATTR_PID,
    NL80211_ATTR_4ADDR,
    NL80211_ATTR_SURVEY_INFO,
    NL80211_ATTR_PMKID,
    NL80211_ATTR_MAX_NUM_PMKIDS,
    NL80211_ATTR_DURATION,
    NL80211_ATTR_COOKIE,
    NL80211_ATTR_WIPHY_COVERAGE_CLASS,
    NL80211_ATTR_TX_RATES,
    NL80211_ATTR_FRAME_MATCH,
    NL80211_ATTR_ACK,
    NL80211_ATTR_PS_STATE,
    NL80211_ATTR_CQM,
    NL80211_ATTR_LOCAL_STATE_CHANGE,
    NL80211_ATTR_AP_ISOLATE,
    NL80211_ATTR_WIPHY_TX_POWER_SETTING,
    NL80211_ATTR_WIPHY_TX_POWER_LEVEL,
    NL80211_ATTR_TX_FRAME_TYPES,
    NL80211_ATTR_RX_FRAME_TYPES,
    NL80211_ATTR_FRAME_TYPE,
    NL80211_ATTR_CONTROL_PORT_ETHERTYPE,
    NL80211_ATTR_CONTROL_PORT_NO_ENCRYPT,
    NL80211_ATTR_SUPPORT_IBSS_RSN,
    NL80211_ATTR_WIPHY_ANTENNA_TX,
    NL80211_ATTR_WIPHY_ANTENNA_RX,
    NL80211_ATTR_MCAST_RATE,
    NL80211_ATTR_OFFCHANNEL_TX_OK,
    NL80211_ATTR_BSS_HT_OPMODE,
    NL80211_ATTR_KEY_DEFAULT_TYPES,
    NL80211_ATTR_MAX_REMAIN_ON_CHANNEL_DURATION,
    NL80211_ATTR_MESH_SETUP,
    NL80211_ATTR_WIPHY_ANTENNA_AVAIL_TX,
    NL80211_ATTR_WIPHY_ANTENNA_AVAIL_RX,
    NL80211_ATTR_SUPPORT_MESH_AUTH,
    NL80211_ATTR_STA_PLINK_STATE,
    NL80211_ATTR_WOWLAN_TRIGGERS,
    NL80211_ATTR_WOWLAN_TRIGGERS_SUPPORTED,
    NL80211_ATTR_SCHED_SCAN_INTERVAL,
    NL80211_ATTR_INTERFACE_COMBINATIONS,
    NL80211_ATTR_SOFTWARE_IFTYPES,
    NL80211_ATTR_REKEY_DATA,
    NL80211_ATTR_MAX_NUM_SCHED_SCAN_SSIDS,
    NL80211_ATTR_MAX_SCHED_SCAN_IE_LEN,
    NL80211_ATTR_SCAN_SUPP_RATES,
    NL80211_ATTR_HIDDEN_SSID,
    NL80211_ATTR_IE_PROBE_RESP,
    NL80211_ATTR_IE_ASSOC_RESP,
    NL80211_ATTR_STA_WME,
    NL80211_ATTR_SUPPORT_AP_UAPSD,
    NL80211_ATTR_ROAM_SUPPORT,
    NL80211_ATTR_SCHED_SCAN_MATCH,
    NL80211_ATTR_MAX_MATCH_SETS,
    NL80211_ATTR_PMKSA_CANDIDATE,
    NL80211_ATTR_TX_NO_CCK_RATE,
    NL80211_ATTR_TDLS_ACTION,
    NL80211_ATTR_TDLS_DIALOG_TOKEN,
    NL80211_ATTR_TDLS_OPERATION,
    NL80211_ATTR_TDLS_SUPPORT,
    NL80211_ATTR_TDLS_EXTERNAL_SETUP,
    NL80211_ATTR_DEVICE_AP_SME,
    NL80211_ATTR_DONT_WAIT_FOR_ACK,
    NL80211_ATTR_FEATURE_FLAGS,
    NL80211_ATTR_PROBE_RESP_OFFLOAD,
    NL80211_ATTR_PROBE_RESP,
    NL80211_ATTR_DFS_REGION,
    NL80211_ATTR_DISABLE_HT,
    NL80211_ATTR_HT_CAPABILITY_MASK,
    NL80211_ATTR_NOACK_MAP,
    NL80211_ATTR_INACTIVITY_TIMEOUT,
    NL80211_ATTR_RX_SIGNAL_DBM,
    NL80211_ATTR_BG_SCAN_PERIOD,
    NL80211_ATTR_WDEV,
    NL80211_ATTR_USER_REG_HINT_TYPE,
    NL80211_ATTR_CONN_FAILED_REASON,
    NL80211_ATTR_AUTH_DATA,
    NL80211_ATTR_VHT_CAPABILITY,
    NL80211_ATTR_SCAN_FLAGS,
    NL80211_ATTR_CHANNEL_WIDTH,
    NL80211_ATTR_CENTER_FREQ1,
    NL80211_ATTR_CENTER_FREQ2,
    NL80211_ATTR_P2P_CTWINDOW,
    NL80211_ATTR_P2P_OPPPS,
    NL80211_ATTR_LOCAL_MESH_POWER_MODE,
    NL80211_ATTR_ACL_POLICY,
    NL80211_ATTR_MAC_ADDRS,
    NL80211_ATTR_MAC_ACL_MAX,
    NL80211_ATTR_RADAR_EVENT,
    NL80211_ATTR_EXT_CAPA,
    NL80211_ATTR_EXT_CAPA_MASK,
    NL80211_ATTR_STA_CAPABILITY,
    NL80211_ATTR_STA_EXT_CAPABILITY,
    NL80211_ATTR_PROTOCOL_FEATURES,
    NL80211_ATTR_SPLIT_WIPHY_DUMP,
    NL80211_ATTR_DISABLE_VHT,
    NL80211_ATTR_VHT_CAPABILITY_MASK,
    NL80211_ATTR_MDID,
    NL80211_ATTR_IE_RIC,
    NL80211_ATTR_CRIT_PROT_ID,
    NL80211_ATTR_MAX_CRIT_PROT_DURATION,
    NL80211_ATTR_PEER_AID,
    NL80211_ATTR_COALESCE_RULE,
    NL80211_ATTR_CH_SWITCH_COUNT,
    NL80211_ATTR_CH_SWITCH_BLOCK_TX,
    NL80211_ATTR_CSA_IES,
    NL80211_ATTR_CSA_C_OFF_BEACON,
    NL80211_ATTR_CSA_C_OFF_PRESP,
    NL80211_ATTR_RXMGMT_FLAGS,
    NL80211_ATTR_STA_SUPPORTED_CHANNELS,
    NL80211_ATTR_STA_SUPPORTED_OPER_CLASSES,
    NL80211_ATTR_HANDLE_DFS,
    NL80211_ATTR_SUPPORT_5_MHZ,
    NL80211_ATTR_SUPPORT_10_MHZ,
    NL80211_ATTR_OPMODE_NOTIF,
    NL80211_ATTR_VENDOR_ID,
    NL80211_ATTR_VENDOR_SUBCMD,
    NL80211_ATTR_VENDOR_DATA,
    NL80211_ATTR_VENDOR_EVENTS,
    NL80211_ATTR_QOS_MAP,
    NL80211_ATTR_MAC_HINT,
    NL80211_ATTR_WIPHY_FREQ_HINT,
    NL80211_ATTR_MAX_AP_ASSOC_STA,
    NL80211_ATTR_TDLS_PEER_CAPABILITY,
    NL80211_ATTR_SOCKET_OWNER,
    NL80211_ATTR_CSA_C_OFFSETS_TX,
    NL80211_ATTR_MAX_CSA_COUNTERS,
    NL80211_ATTR_TDLS_INITIATOR,
    NL80211_ATTR_USE_RRM,
    NL80211_ATTR_WIPHY_DYN_ACK,
    NL80211_ATTR_TSID,
    NL80211_ATTR_USER_PRIO,
    NL80211_ATTR_ADMITTED_TIME,
    NL80211_ATTR_SMPS_MODE,
    NL80211_ATTR_OPER_CLASS,
    NL80211_ATTR_MAC_MASK,
    NL80211_ATTR_WIPHY_SELF_MANAGED_REG,
    NL80211_ATTR_EXT_FEATURES,
    NL80211_ATTR_SURVEY_RADIO_STATS,
    NL80211_ATTR_NETNS_FD,
    NL80211_ATTR_SCHED_SCAN_DELAY,
    NL80211_ATTR_REG_INDOOR,
    NL80211_ATTR_MAX_NUM_SCHED_SCAN_PLANS,
    NL80211_ATTR_MAX_SCAN_PLAN_INTERVAL,
    NL80211_ATTR_MAX_SCAN_PLAN_ITERATIONS,
    NL80211_ATTR_SCHED_SCAN_PLANS,
    NL80211_ATTR_PBSS,
    NL80211_ATTR_BSS_SELECT,
    NL80211_ATTR_STA_SUPPORT_P2P_PS,
    NL80211_ATTR_PAD,
    NL80211_ATTR_IFTYPE_EXT_CAPA,
    NL80211_ATTR_MU_MIMO_GROUP_DATA,
    NL80211_ATTR_MU_MIMO_FOLLOW_MAC_ADDR,
    NL80211_ATTR_SCAN_START_TIME_TSF,
    NL80211_ATTR_SCAN_START_TIME_TSF_BSSID,
    NL80211_ATTR_MEASUREMENT_DURATION,
    NL80211_ATTR_MEASUREMENT_DURATION_MANDATORY,
    NL80211_ATTR_MESH_PEER_AID,
    NL80211_ATTR_NAN_MASTER_PREF,
    NL80211_ATTR_BANDS,
    NL80211_ATTR_NAN_FUNC,
    NL80211_ATTR_NAN_MATCH,
    NL80211_ATTR_FILS_KEK,
    NL80211_ATTR_FILS_NONCES,
    NL80211_ATTR_MULTICAST_TO_UNICAST_ENABLED,
    NL80211_ATTR_BSSID,
    NL80211_ATTR_SCHED_SCAN_RELATIVE_RSSI,
    NL80211_ATTR_SCHED_SCAN_RSSI_ADJUST,
    NL80211_ATTR_TIMEOUT_REASON,
    NL80211_ATTR_FILS_ERP_USERNAME,
    NL80211_ATTR_FILS_ERP_REALM,
    NL80211_ATTR_FILS_ERP_NEXT_SEQ_NUM,
    NL80211_ATTR_FILS_ERP_RRK,
    NL80211_ATTR_FILS_CACHE_ID,
    NL80211_ATTR_PMK,
    NL80211_ATTR_SCHED_SCAN_MULTI,
    NL80211_ATTR_SCHED_SCAN_MAX_REQS,
    NL80211_ATTR_WANT_1X_4WAY_HS,
    NL80211_ATTR_PMKR0_NAME,
    NL80211_ATTR_PORT_AUTHORIZED,
    NL80211_ATTR_EXTERNAL_AUTH_ACTION,
    NL80211_ATTR_EXTERNAL_AUTH_SUPPORT,
    NL80211_ATTR_NSS,
    NL80211_ATTR_ACK_SIGNAL,
    NL80211_ATTR_CONTROL_PORT_OVER_NL80211,
    NUM_NL80211_ATTR
}
static NL80211_ATTR_MAX: usize = nl80211_attr::NUM_NL80211_ATTR as usize - 1;

#[repr(C)]
enum nl80211_bss {
    __NL80211_BSS_INVALID,
    NL80211_BSS_BSSID,
    NL80211_BSS_FREQUENCY,
    NL80211_BSS_TSF,
    NL80211_BSS_BEACON_INTERVAL,
    NL80211_BSS_CAPABILITY,
    NL80211_BSS_INFORMATION_ELEMENTS,
    NL80211_BSS_SIGNAL_MBM,
    NL80211_BSS_SIGNAL_UNSPEC,
    NL80211_BSS_STATUS,
    NL80211_BSS_SEEN_MS_AGO,
    NL80211_BSS_BEACON_IES,
    NL80211_BSS_CHAN_WIDTH,
    NL80211_BSS_BEACON_TSF,
    NL80211_BSS_PRESP_DATA,
    NL80211_BSS_LAST_SEEN_BOOTTIME,
    NL80211_BSS_PAD,
    NL80211_BSS_PARENT_TSF,
    NL80211_BSS_PARENT_BSSID,
    NL80211_BSS_CHAIN_SIGNAL,
    NUM_NL80211_BSS
}
static NL80211_BSS_MAX: usize = nl80211_bss::NUM_NL80211_BSS as usize - 1;

#[repr(C)]
enum nl80211_bss_status {
    NL80211_BSS_STATUS_AUTHENTICATED,
    NL80211_BSS_STATUS_ASSOCIATED,
    NL80211_BSS_STATUS_IBSS_JOINED,
}

#[repr(C)]
enum nl80211_commands {
    NL80211_CMD_UNSPEC,
    NL80211_CMD_GET_WIPHY,
    NL80211_CMD_SET_WIPHY,
    NL80211_CMD_NEW_WIPHY,
    NL80211_CMD_DEL_WIPHY,
    NL80211_CMD_GET_INTERFACE,
    NL80211_CMD_SET_INTERFACE,
    NL80211_CMD_NEW_INTERFACE,
    NL80211_CMD_DEL_INTERFACE,
    NL80211_CMD_GET_KEY,
    NL80211_CMD_SET_KEY,
    NL80211_CMD_NEW_KEY,
    NL80211_CMD_DEL_KEY,
    NL80211_CMD_GET_BEACON,
    NL80211_CMD_SET_BEACON,
    NL80211_CMD_START_AP, // NL80211_CMD_NEW_BEACON = NL80211_CMD_START_AP,
    NL80211_CMD_STOP_AP, // NL80211_CMD_DEL_BEACON = NL80211_CMD_STOP_AP,
    NL80211_CMD_GET_STATION,
    NL80211_CMD_SET_STATION,
    NL80211_CMD_NEW_STATION,
    NL80211_CMD_DEL_STATION,
    NL80211_CMD_GET_MPATH,
    NL80211_CMD_SET_MPATH,
    NL80211_CMD_NEW_MPATH,
    NL80211_CMD_DEL_MPATH,
    NL80211_CMD_SET_BSS,
    NL80211_CMD_SET_REG,
    NL80211_CMD_REQ_SET_REG,
    NL80211_CMD_GET_MESH_CONFIG,
    NL80211_CMD_SET_MESH_CONFIG,
    NL80211_CMD_SET_MGMT_EXTRA_IE,
    NL80211_CMD_GET_REG,
    NL80211_CMD_GET_SCAN,
    NL80211_CMD_TRIGGER_SCAN,
    NL80211_CMD_NEW_SCAN_RESULTS,
    NL80211_CMD_SCAN_ABORTED,
    NL80211_CMD_REG_CHANGE,
    NL80211_CMD_AUTHENTICATE,
    NL80211_CMD_ASSOCIATE,
    NL80211_CMD_DEAUTHENTICATE,
    NL80211_CMD_DISASSOCIATE,
    NL80211_CMD_MICHAEL_MIC_FAILURE,
    NL80211_CMD_REG_BEACON_HINT,
    NL80211_CMD_JOIN_IBSS,
    NL80211_CMD_LEAVE_IBSS,
    NL80211_CMD_TESTMODE,
    NL80211_CMD_CONNECT,
    NL80211_CMD_ROAM,
    NL80211_CMD_DISCONNECT,
    NL80211_CMD_SET_WIPHY_NETNS,
    NL80211_CMD_GET_SURVEY,
    NL80211_CMD_NEW_SURVEY_RESULTS,
    NL80211_CMD_SET_PMKSA,
    NL80211_CMD_DEL_PMKSA,
    NL80211_CMD_FLUSH_PMKSA,
    NL80211_CMD_REMAIN_ON_CHANNEL,
    NL80211_CMD_CANCEL_REMAIN_ON_CHANNEL,
    NL80211_CMD_SET_TX_BITRATE_MASK,
    NL80211_CMD_REGISTER_FRAME, // NL80211_CMD_REGISTER_ACTION = NL80211_CMD_REGISTER_FRAME,
    NL80211_CMD_FRAME, // NL80211_CMD_ACTION = NL80211_CMD_FRAME,
    NL80211_CMD_FRAME_TX_STATUS, // NL80211_CMD_ACTION_TX_STATUS = NL80211_CMD_FRAME_TX_STATUS,
    NL80211_CMD_SET_POWER_SAVE,
    NL80211_CMD_GET_POWER_SAVE,
    NL80211_CMD_SET_CQM,
    NL80211_CMD_NOTIFY_CQM,
    NL80211_CMD_SET_CHANNEL,
    NL80211_CMD_SET_WDS_PEER,
    NL80211_CMD_FRAME_WAIT_CANCEL,
    NL80211_CMD_JOIN_MESH,
    NL80211_CMD_LEAVE_MESH,
    NL80211_CMD_UNPROT_DEAUTHENTICATE,
    NL80211_CMD_UNPROT_DISASSOCIATE,
    NL80211_CMD_NEW_PEER_CANDIDATE,
    NL80211_CMD_GET_WOWLAN,
    NL80211_CMD_SET_WOWLAN,
    NL80211_CMD_START_SCHED_SCAN,
    NL80211_CMD_STOP_SCHED_SCAN,
    NL80211_CMD_SCHED_SCAN_RESULTS,
    NL80211_CMD_SCHED_SCAN_STOPPED,
    NL80211_CMD_SET_REKEY_OFFLOAD,
    NL80211_CMD_PMKSA_CANDIDATE,
    NL80211_CMD_TDLS_OPER,
    NL80211_CMD_TDLS_MGMT,
    NL80211_CMD_UNEXPECTED_FRAME,
    NL80211_CMD_PROBE_CLIENT,
    NL80211_CMD_REGISTER_BEACONS,
    NL80211_CMD_UNEXPECTED_4ADDR_FRAME,
    NL80211_CMD_SET_NOACK_MAP,
    NL80211_CMD_CH_SWITCH_NOTIFY,
    NL80211_CMD_START_P2P_DEVICE,
    NL80211_CMD_STOP_P2P_DEVICE,
    NL80211_CMD_CONN_FAILED,
    NL80211_CMD_SET_MCAST_RATE,
    NL80211_CMD_SET_MAC_ACL,
    NL80211_CMD_RADAR_DETECT,
    NL80211_CMD_GET_PROTOCOL_FEATURES,
    NL80211_CMD_UPDATE_FT_IES,
    NL80211_CMD_FT_EVENT,
    NL80211_CMD_CRIT_PROTOCOL_START,
    NL80211_CMD_CRIT_PROTOCOL_STOP,
    NL80211_CMD_GET_COALESCE,
    NL80211_CMD_SET_COALESCE,
    NL80211_CMD_CHANNEL_SWITCH,
    NL80211_CMD_VENDOR,
    NL80211_CMD_SET_QOS_MAP,
    NL80211_CMD_ADD_TX_TS,
    NL80211_CMD_DEL_TX_TS,
    NL80211_CMD_GET_MPP,
    NL80211_CMD_JOIN_OCB,
    NL80211_CMD_LEAVE_OCB,
    NL80211_CMD_CH_SWITCH_STARTED_NOTIFY,
    NL80211_CMD_TDLS_CHANNEL_SWITCH,
    NL80211_CMD_TDLS_CANCEL_CHANNEL_SWITCH,
    NL80211_CMD_WIPHY_REG_CHANGE,
    NL80211_CMD_ABORT_SCAN,
    NL80211_CMD_START_NAN,
    NL80211_CMD_STOP_NAN,
    NL80211_CMD_ADD_NAN_FUNCTION,
    NL80211_CMD_DEL_NAN_FUNCTION,
    NL80211_CMD_CHANGE_NAN_CONFIG,
    NL80211_CMD_NAN_MATCH,
    NL80211_CMD_SET_MULTICAST_TO_UNICAST,
    NL80211_CMD_UPDATE_CONNECT_PARAMS,
    NL80211_CMD_SET_PMK,
    NL80211_CMD_DEL_PMK,
    NL80211_CMD_PORT_AUTHORIZED,
    NL80211_CMD_RELOAD_REGDB,
    NL80211_CMD_EXTERNAL_AUTH,
    NL80211_CMD_STA_OPMODE_CHANGED,
    NL80211_CMD_CONTROL_PORT_FRAME,
    NUM_NL80211_CMD
}
static NL80211_CMD_MAX: usize = nl80211_commands::NUM_NL80211_CMD as usize - 1;

#[repr(C)]
enum nl80211_rate_info {
    __NL80211_RATE_INFO_INVALID,
    NL80211_RATE_INFO_BITRATE,
    NL80211_RATE_INFO_MCS,
    NL80211_RATE_INFO_40_MHZ_WIDTH,
    NL80211_RATE_INFO_SHORT_GI,
    NL80211_RATE_INFO_BITRATE32,
    NL80211_RATE_INFO_VHT_MCS,
    NL80211_RATE_INFO_VHT_NSS,
    NL80211_RATE_INFO_80_MHZ_WIDTH,
    NL80211_RATE_INFO_80P80_MHZ_WIDTH,
    NL80211_RATE_INFO_160_MHZ_WIDTH,
    NL80211_RATE_INFO_10_MHZ_WIDTH,
    NL80211_RATE_INFO_5_MHZ_WIDTH,
    NUM_NL80211_RATE_INFO
}
static NL80211_RATE_INFO_MAX: usize = nl80211_rate_info::NUM_NL80211_RATE_INFO as usize - 1;

#[repr(C)]
enum nl80211_sta_info {
    __NL80211_STA_INFO_INVALID,
    NL80211_STA_INFO_INACTIVE_TIME,
    NL80211_STA_INFO_RX_BYTES,
    NL80211_STA_INFO_TX_BYTES,
    NL80211_STA_INFO_LLID,
    NL80211_STA_INFO_PLID,
    NL80211_STA_INFO_PLINK_STATE,
    NL80211_STA_INFO_SIGNAL,
    NL80211_STA_INFO_TX_BITRATE,
    NL80211_STA_INFO_RX_PACKETS,
    NL80211_STA_INFO_TX_PACKETS,
    NL80211_STA_INFO_TX_RETRIES,
    NL80211_STA_INFO_TX_FAILED,
    NL80211_STA_INFO_SIGNAL_AVG,
    NL80211_STA_INFO_RX_BITRATE,
    NL80211_STA_INFO_BSS_PARAM,
    NL80211_STA_INFO_CONNECTED_TIME,
    NL80211_STA_INFO_STA_FLAGS,
    NL80211_STA_INFO_BEACON_LOSS,
    NL80211_STA_INFO_T_OFFSET,
    NL80211_STA_INFO_LOCAL_PM,
    NL80211_STA_INFO_PEER_PM,
    NL80211_STA_INFO_NONPEER_PM,
    NL80211_STA_INFO_RX_BYTES64,
    NL80211_STA_INFO_TX_BYTES64,
    NL80211_STA_INFO_CHAIN_SIGNAL,
    NL80211_STA_INFO_CHAIN_SIGNAL_AVG,
    NL80211_STA_INFO_EXPECTED_THROUGHPUT,
    NL80211_STA_INFO_RX_DROP_MISC,
    NL80211_STA_INFO_BEACON_RX,
    NL80211_STA_INFO_BEACON_SIGNAL_AVG,
    NL80211_STA_INFO_TID_STATS,
    NL80211_STA_INFO_RX_DURATION,
    NL80211_STA_INFO_PAD,
    NL80211_STA_INFO_ACK_SIGNAL,
    NUM_NL80211_STA_INFO
}
static NL80211_STA_INFO_MAX: usize = nl80211_sta_info::NUM_NL80211_STA_INFO as usize - 1;

static NL_AUTO_PORT: u32 = 0;
static NL_AUTO_SEQ: u32 = 0;
static NLM_F_ROOT: i32 = 0x100;
static NLM_F_MATCH: i32 = 0x200;
static NLM_F_DUMP: i32 = NLM_F_ROOT | NLM_F_MATCH;

#[repr(C)] struct genlmsghdr {}
#[repr(C)] struct nlmsghdr {}
#[repr(C)] struct nl_msg {}
#[repr(C)] struct nl_sock {}
#[repr(C)] struct nlattr {}

#[repr(C)]
#[derive(Copy, Clone)]
struct nla_policy {
    type_: u16, // Type of attribute or NLA_UNSPEC
    minlen: u16, // Minimal length of payload required
    maxlen: u16, // Maximal length of payload allowed
}

struct NlSocket {
    ptr: *mut nl_sock,
}

impl NlSocket {
    fn new() -> Self {
        let result = unsafe { nl_socket_alloc() };
        if result.is_null() {
            panic!("nl_socket_alloc failed");
        }
        Self { ptr: result }
    }

    fn connect(&self) {
        let ret = unsafe { genl_connect(self.ptr) };
        if ret < 0 {
            panic!("genl_connect returned {}", ret);
        }
    }

    fn modify_cb(&self, type_: nl_cb_type, kind: nl_cb_kind, func: extern fn(*mut nl_msg, arg: *mut WLanInfo) -> c_int, arg: *mut WLanInfo) {
        let ret = unsafe { nl_socket_modify_cb(self.ptr, type_, kind, func, arg) };
        if ret < 0 {
            panic!("nl_socket_modify_cb returned {}", ret);
        }
    }

    fn send_sync(&self, msg: NlMsg) {
        let ret = unsafe { nl_send_sync(self.ptr, msg.ptr) };
        mem::forget(msg); // nl_send_sync frees the msg internally
        if ret < 0 {
            panic!("nl_send_sync returned {}", ret);
        }
    }

    fn ctrl_resolve(&self, name: &str) -> c_int {
        let name_c = ffi::CString::new(name).unwrap();
        let ret = unsafe { genl_ctrl_resolve(self.ptr, name_c.as_ptr()) };
        if ret < 0 {
            panic!("genl_ctrl_resolve returned {}", ret);
        }
        ret
    }
}

impl Drop for NlSocket {
    fn drop(&mut self) {
        unsafe { nl_socket_free(self.ptr) };
    }
}

struct NlMsg {
    ptr: *mut nl_msg,
}

impl NlMsg {
    fn new() -> Self {
        let result = unsafe { nlmsg_alloc() };
        if result.is_null() {
            panic!("nlmsg_alloc failed");
        }
        Self { ptr: result }
    }

    fn genl_put(&self, port: u32, seq: u32, family: c_int, hdrlen: c_int, flags: c_int, cmd: u8, version: u8) {
        let ret = unsafe { genlmsg_put(self.ptr, port, seq, family, hdrlen, flags, cmd, version) };
        if ret.is_null() {
            panic!("genlmsg_put returned NULL");
        }
    }

    fn attr_put(&self, attrtype: c_int, datalen: c_int, data: *const c_void) {
        let ret = unsafe { nla_put(self.ptr, attrtype, datalen, data) };
        if ret < 0 {
            panic!("nla_put returned {}", ret);
        }
    }

    fn attr_put_u32(&self, attrtype: c_int, value: u32) {
        let ret = unsafe { nla_put_u32(self.ptr, attrtype, value) };
        if ret < 0 {
            panic!("nla_put_u32 returned {}", ret);
        }
    }
}

impl Drop for NlMsg {
    fn drop(&mut self) {
        unsafe { nlmsg_free(self.ptr) };
    }
}

struct NlMsgHdrData {
    ptr: *const genlmsghdr,
}

impl NlMsgHdrData {
    fn new(msg: *mut nl_msg) -> Self {
        NlMsgHdrData { ptr: unsafe { nlmsg_data(nlmsg_hdr(msg)) } as *const genlmsghdr }
    }

    fn parse_attrs(&self, n: usize) -> Option<Vec<*mut nlattr>> {
        let mut ret: Vec<*mut nlattr> = Vec::with_capacity(n + 1);
        unsafe { ret.set_len(n + 1); }

        if unsafe { nla_parse(ret.as_mut_ptr(), n as i32, genlmsg_attrdata(self.ptr, 0), genlmsg_attrlen(self.ptr, 0), ptr::null_mut()) } < 0 {
            None
        } else {
            Some(ret)
        }
    }
}

fn parse_attrs(n: usize, data: *mut nlattr, policy: NlAttrPolicy) -> Option<Vec<*mut nlattr>> {
    let mut ret: Vec<*mut nlattr> = Vec::with_capacity(n + 1);
    unsafe { ret.set_len(n + 1); }
    let mut policy = policy;

    if unsafe { nla_parse_nested(ret.as_mut_ptr(), n as i32, data, policy.as_mut_ptr()) } < 0 {
        None
    } else {
        Some(ret)
    }
}

struct NlAttrPolicy {
    policy: Vec<nla_policy>,
}

impl NlAttrPolicy {
    fn new(size: usize) -> Self {
        let mut result = Self { policy: Vec::with_capacity(size) };
        result.policy.resize(size, nla_policy { type_: 0, minlen: 0, maxlen: 0 });
        result
    }

    fn set_type(&mut self, value: usize, type_: nla_types) {
        self.policy[value].type_ = type_ as u16;
    }

    fn as_mut_ptr(&mut self) -> *mut nla_policy {
        self.policy.as_mut_ptr()
    }
}

// Based on NetworkManager/src/platform/wifi/wifi-utils-nl80211.c
fn nl80211_xbm_to_percent(xbm: f64) -> f64 {
    static NOISE_FLOOR_DBM: f64 = -90.0;
    static SIGNAL_MAX_DBM: f64 = -20.0;

    let mut xbm = xbm / 100.0;
    if xbm < NOISE_FLOOR_DBM {
        xbm = NOISE_FLOOR_DBM;
    }
    if xbm > SIGNAL_MAX_DBM {
        xbm = SIGNAL_MAX_DBM;
    }

    (100.0 - 70.0 * ((SIGNAL_MAX_DBM - xbm) / (SIGNAL_MAX_DBM - NOISE_FLOOR_DBM)))
}

// Based on NetworkManager/src/platform/wifi/wifi-utils-nl80211.c
fn find_ssid(ies: &[u8]) -> Option<String> {
    static WLAN_EID_SSID: u8 = 0;
    let mut ies = ies;

    while ies.len() > 2 && ies[0] != WLAN_EID_SSID {
        ies = &ies[2..];
    }
    if ies.len() < 2 || ies.len() < (ies[1] + 2) as usize {
        None
    } else {
        let ssid = &ies[2..ies[1] as usize + 2];
        Some(String::from_utf8_lossy(ssid).into_owned())
    }
}

extern "C" fn gwi_sta_cb(msg: *mut nl_msg, data: *mut WLanInfo) -> c_int {
    unsafe { data.as_mut() }.unwrap().handle_station_cb(NlMsgHdrData::new(msg)) as c_int
}

extern "C" fn gwi_scan_cb(msg: *mut nl_msg, data: *mut WLanInfo) -> c_int {
    unsafe { data.as_mut() }.unwrap().handle_scan_cb(NlMsgHdrData::new(msg)) as c_int
}

impl WLanInfo {
    pub fn new(interface: &str) -> Self {
        let mut result = Self { essid: None, bssid: [0; ETH_ALEN as usize], quality: None, bitrate: None, frequency: None };

        let sk = NlSocket::new();
        sk.connect();

        sk.modify_cb(nl_cb_type::NL_CB_VALID, nl_cb_kind::NL_CB_CUSTOM, gwi_scan_cb, &mut result);
        let nl80211_id = sk.ctrl_resolve("nl80211");

        let iface_c = ffi::CString::new(interface).unwrap();
        let ifidx = unsafe { if_nametoindex(iface_c.as_ptr()) };
        if ifidx == 0 {
            panic!("if_nametoindex returned 0");
        }

        let mut msg = NlMsg::new();
        msg.genl_put(NL_AUTO_PORT, NL_AUTO_SEQ, nl80211_id, 0, NLM_F_DUMP, nl80211_commands::NL80211_CMD_GET_SCAN as u8, 0);
        msg.attr_put_u32(nl80211_attr::NL80211_ATTR_IFINDEX as i32, ifidx);
        sk.send_sync(msg);

        sk.modify_cb(nl_cb_type::NL_CB_VALID, nl_cb_kind::NL_CB_CUSTOM, gwi_sta_cb, &mut result);

        msg = NlMsg::new();
        msg.genl_put(NL_AUTO_PORT, NL_AUTO_SEQ, nl80211_id, 0, NLM_F_DUMP, nl80211_commands::NL80211_CMD_GET_STATION as u8, 0);
        msg.attr_put_u32(nl80211_attr::NL80211_ATTR_IFINDEX as i32, ifidx);
        msg.attr_put(nl80211_attr::NL80211_ATTR_MAC as i32, result.bssid.len() as i32, &result.bssid as *const _ as *const c_void);
        sk.send_sync(msg);

        result
    }

    fn handle_scan_cb(&mut self, data: NlMsgHdrData) -> nl_cb_action {
        let bss = {
            let bss_attr = {
                let tb = match data.parse_attrs(NL80211_ATTR_MAX) {
                    Some(tb) => tb,
                    None => { return nl_cb_action::NL_SKIP; },
                };

                if tb[nl80211_attr::NL80211_ATTR_BSS as usize].is_null() {
                    return nl_cb_action::NL_SKIP;
                }

                tb[nl80211_attr::NL80211_ATTR_BSS as usize]
            };

            let mut bss_policy = NlAttrPolicy::new(NL80211_BSS_MAX + 1);
            bss_policy.set_type(nl80211_bss::NL80211_BSS_FREQUENCY as usize, nla_types::NLA_U32);
            bss_policy.set_type(nl80211_bss::NL80211_BSS_BSSID as usize, nla_types::NLA_UNSPEC);
            bss_policy.set_type(nl80211_bss::NL80211_BSS_INFORMATION_ELEMENTS as usize, nla_types::NLA_UNSPEC);
            bss_policy.set_type(nl80211_bss::NL80211_BSS_SIGNAL_MBM as usize, nla_types::NLA_U32);
            bss_policy.set_type(nl80211_bss::NL80211_BSS_SIGNAL_UNSPEC as usize, nla_types::NLA_U8);
            bss_policy.set_type(nl80211_bss::NL80211_BSS_STATUS as usize, nla_types::NLA_U32);

            match parse_attrs(NL80211_BSS_MAX, bss_attr, bss_policy) {
                Some(bss) => {
                    if bss[nl80211_bss::NL80211_BSS_STATUS as usize].is_null() {
                        return nl_cb_action::NL_SKIP;
                    }

                    let status = unsafe { nla_get_u32(bss[nl80211_bss::NL80211_BSS_STATUS as usize]) };
                    if status != nl80211_bss_status::NL80211_BSS_STATUS_ASSOCIATED as u32 && status != nl80211_bss_status::NL80211_BSS_STATUS_IBSS_JOINED as u32 {
                        return nl_cb_action::NL_SKIP;
                    }

                    if bss[nl80211_bss::NL80211_BSS_BSSID as usize].is_null() {
                        return nl_cb_action::NL_SKIP;
                    }
                    bss
                },
                None => { return nl_cb_action::NL_SKIP; },
            }
        };

        // bssid is used by NL80211_CMD_GET_STATION
        self.bssid.copy_from_slice(unsafe { slice::from_raw_parts(nla_data(bss[nl80211_bss::NL80211_BSS_BSSID as usize]) as *const u8, ETH_ALEN as usize) });

        if !bss[nl80211_bss::NL80211_BSS_FREQUENCY as usize].is_null() {
            self.frequency = Some(unsafe { nla_get_u32(bss[nl80211_bss::NL80211_BSS_FREQUENCY as usize]) } as f64 * 1.0e6);
        }

        if !bss[nl80211_bss::NL80211_BSS_SIGNAL_UNSPEC as usize].is_null() {
            self.quality = Some(unsafe { nla_get_u8(bss[nl80211_bss::NL80211_BSS_SIGNAL_UNSPEC as usize]) } as f64);
        }

        if !bss[nl80211_bss::NL80211_BSS_SIGNAL_MBM as usize].is_null() {
            self.quality = Some(nl80211_xbm_to_percent(unsafe { nla_get_s32(bss[nl80211_bss::NL80211_BSS_SIGNAL_MBM as usize]) } as f64));
        }

        if !bss[nl80211_bss::NL80211_BSS_INFORMATION_ELEMENTS as usize].is_null() {
            let elements = unsafe { slice::from_raw_parts(nla_data(bss[nl80211_bss::NL80211_BSS_INFORMATION_ELEMENTS as usize]) as *const u8,
                                                          nla_len(bss[nl80211_bss::NL80211_BSS_INFORMATION_ELEMENTS as usize]) as usize) };
            self.essid = find_ssid(elements);
        }

        nl_cb_action::NL_SKIP
    }

    fn handle_station_cb(&mut self, data: NlMsgHdrData) -> nl_cb_action {
        let rate_info = {
            let rate_info_attr = {
                let sta_info_attr = {
                    let tb = match data.parse_attrs(NL80211_ATTR_MAX) {
                        Some(tb) => tb,
                        None => { return nl_cb_action::NL_SKIP; },
                    };

                    if tb[nl80211_attr::NL80211_ATTR_STA_INFO as usize].is_null() {
                        return nl_cb_action::NL_SKIP;
                    }
                    tb[nl80211_attr::NL80211_ATTR_STA_INFO as usize]
                };

                let mut stats_policy = NlAttrPolicy::new(NL80211_STA_INFO_MAX + 1);
                stats_policy.set_type(nl80211_sta_info::NL80211_STA_INFO_RX_BITRATE as usize, nla_types::NLA_NESTED);

                let sinfo = match parse_attrs(NL80211_STA_INFO_MAX, sta_info_attr, stats_policy) {
                    Some(sinfo) => sinfo,
                    None => { return nl_cb_action::NL_SKIP; },
                };
                if sinfo[nl80211_sta_info::NL80211_STA_INFO_RX_BITRATE as usize].is_null() {
                    return nl_cb_action::NL_SKIP;
                }
                sinfo[nl80211_sta_info::NL80211_STA_INFO_RX_BITRATE as usize]
            };

            let mut rate_policy = NlAttrPolicy::new(NL80211_RATE_INFO_MAX + 1);
            rate_policy.set_type(nl80211_rate_info::NL80211_RATE_INFO_BITRATE as usize, nla_types::NLA_U16);

            match parse_attrs(NL80211_RATE_INFO_MAX, rate_info_attr, rate_policy) {
                Some(rinfo) => rinfo,
                None => { return nl_cb_action::NL_SKIP; },
            }
        };

        if !rate_info[nl80211_rate_info::NL80211_RATE_INFO_BITRATE as usize].is_null() {
            // NL80211_RATE_INFO_BITRATE is specified in units of 100 kbit/s, but iw
            // used to specify bit/s, so we convert to use the same code path.
            self.bitrate = Some(unsafe { nla_get_u16(rate_info[nl80211_rate_info::NL80211_RATE_INFO_BITRATE as usize]) } as f64 * 1.0e5);
        }

        nl_cb_action::NL_SKIP
    }
}
