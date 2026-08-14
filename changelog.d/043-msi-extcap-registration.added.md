The Windows installer can now register fragcap as a Wireshark extcap capture
source. An optional wizard step offers two opt-in choices, both off by default:
register for the current user (the common case), or, for administrators, register
for all users on the machine when Wireshark is detected. Both drive the existing
`fragcap extcap install` command, a failed registration never fails the install,
and leaving both unchecked registers nothing so you can run `fragcap extcap
install` later. The choices are also public installer properties
(`REGISTEREXTCAP_USER`, `REGISTEREXTCAP_MACHINE`) for silent installs.
