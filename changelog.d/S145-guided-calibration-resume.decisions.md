<!-- spec-impact: 15, 17.2.1, 25, 26, 28.1 -->
Guided calibration progress now lives in additive local-store schema version 11 as a
revision-checked checkpoint distinct from compatibility facts and recovery journals.
Its bounded canonical exact-case history carries S144's no-repeat rule across process
exit without treating an authorization refusal as an effectful attempt.
It stores no authorization, secret, trust state, endpoint, or effect authority. A
crash-shaped in-flight row first becomes an interruption pause, and every resumed
attempt still requires a fresh complete plan and response.
