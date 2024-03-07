// SPDX-License-Identifier: Apache-2.0

// SPDX-License-Identifier: MIT

use std::os::unix::io::RawFd;
use std::os::fd::AsRawFd;

use futures::stream::StreamExt;

use netlink_sys::{AsyncSocket, SocketAddr};
use rtnetlink::new_connection;

const RTNLGRP_LINK: u32 = 1;
const RTNLGRP_NEIGH: u32 = 3;
const RTNLGRP_IPV4_IFADDR: u32 = 5;
const RTNLGRP_IPV4_MROUTE: u32 = 6;
const RTNLGRP_IPV4_ROUTE: u32 = 7;
const RTNLGRP_IPV4_RULE: u32 = 8;
const RTNLGRP_IPV6_IFADDR: u32 = 9;
const RTNLGRP_IPV6_MROUTE: u32 = 10;
const RTNLGRP_IPV6_ROUTE: u32 = 11;
const RTNLGRP_IPV6_RULE: u32 = 19;
const RTNLGRP_IPV4_NETCONF: u32 = 24;
const RTNLGRP_IPV6_NETCONF: u32 = 25;
const RTNLGRP_MPLS_ROUTE: u32 = 27;
const RTNLGRP_NSID: u32 = 28;
const RTNLGRP_MPLS_NETCONF: u32 = 29;

const fn nl_mgrp(group: u32) -> u32 {
    if group > 31 {
        panic!("use netlink_sys::Socket::add_membership() for this group");
    }
    if group == 0 {
        0
    } else {
        1 << (group - 1)
    }
}
#[tokio::main]
async fn main() -> Result<(), String> {
    // conn - `Connection` that has a netlink socket which is a `Future` that
    // polls the socket and thus must have an event loop
    //
    // handle - `Handle` to the `Connection`. Used to send/recv netlink
    // messages.
    //
    // messages - A channel receiver.
    let (mut conn, mut _handle, mut messages) = new_connection().map_err(|e| format!("{e}"))?;

    enable_kernel_strict_check(conn.socket_mut().as_raw_fd());

    // These flags specify what kinds of broadcast messages we want to listen
    // for.
    let groups = nl_mgrp(RTNLGRP_LINK)
        | nl_mgrp(RTNLGRP_IPV4_IFADDR)
        | nl_mgrp(RTNLGRP_IPV6_IFADDR)
        | nl_mgrp(RTNLGRP_IPV4_ROUTE)
        | nl_mgrp(RTNLGRP_IPV6_ROUTE)
        | nl_mgrp(RTNLGRP_MPLS_ROUTE)
        | nl_mgrp(RTNLGRP_IPV4_MROUTE)
        | nl_mgrp(RTNLGRP_IPV6_MROUTE)
        | nl_mgrp(RTNLGRP_NEIGH)
        | nl_mgrp(RTNLGRP_IPV4_NETCONF)
        | nl_mgrp(RTNLGRP_IPV6_NETCONF)
        | nl_mgrp(RTNLGRP_IPV4_RULE)
        | nl_mgrp(RTNLGRP_IPV6_RULE)
        | nl_mgrp(RTNLGRP_NSID)
        | nl_mgrp(RTNLGRP_MPLS_NETCONF);

    let addr = SocketAddr::new(0, groups);
    conn.socket_mut()
        .socket_mut()
        .bind(&addr)
        .expect("Failed to bind");

    // Spawn `Connection` to start polling netlink socket.
    tokio::spawn(conn);

    // Start receiving events through `messages` channel.
    while let Some((message, _)) = messages.next().await {
        let payload = message.payload;
        println!("{payload:?}");
    }
    Ok(())
}

const NETLINK_GET_STRICT_CHK: i32 = 12;

fn enable_kernel_strict_check(fd: RawFd) {
    if unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_NETLINK,
            NETLINK_GET_STRICT_CHK,
            1u32.to_ne_bytes().as_ptr() as *const _,
            4,
        )
    } != 0
    {
        println!(
            "Failed to set socket option NETLINK_GET_STRICT_CHK: error {}",
            std::io::Error::last_os_error()
        );
    }
}
