`fragcap extcap install` and `fragcap extcap uninstall` register and unregister
fragcap as a Wireshark extcap capture source, so you no longer copy the binary
into Wireshark's extcap directory by hand; `fragcap doctor` then reports the
integration accordingly. Registration is per user, so on a shared machine each
user who wants the Wireshark integration runs `fragcap extcap install` once; an
administrator can register for the whole machine by pointing `--dir` at
Wireshark's system extcap directory.
