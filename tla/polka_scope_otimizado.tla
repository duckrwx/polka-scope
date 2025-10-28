---- MODULE polka_scope_otimizado ----
EXTENDS Integers, FiniteSets, TLC

(*
   Versão OTIMIZADA do Polka-Scope
   Roda em segundos, não minutos!
*)

CONSTANTS
    MAX_CLOCK,        \* Limite de tempo
    PROBE_INTERVAL    \* Intervalo entre sondagens

(* --algorithm PolkaScopeOtimizado

variables
    (* Estado do agente *)
    status = "Idle",
    
    (* Peers conhecidos - SIMPLIFICADO: apenas conta *)
    num_peers = 0,
    
    (* Resultados - SIMPLIFICADO: apenas sucesso/falha *)
    probe_success = FALSE,
    
    (* Relógio e último fetch *)
    clock = 0,
    last_fetch_time = -10;

define
    (* Invariante de tipo *)
    TypeInvariant ==
        /\ status \in {"Idle", "FetchingPeers", "Probing", "Reporting"}
        /\ clock >= 0
        /\ last_fetch_time >= -10
        /\ num_peers >= 0
        
    (* Propriedade: Eventualmente volta ao Idle *)
    EventuallyIdle == <>(status = "Idle")
    
    (* Propriedade: Se tem peers, eventualmente sonda *)
    EventuallyProbe == (num_peers > 0) ~> (status = "Probing")
end define;

(* Processo do relógio - COM LIMITE *)
fair process Clock = "clock"
begin
    ClockLoop:
        while clock < MAX_CLOCK do
            Tick:
                clock := clock + 1;
        end while;
end process;

(* Processo do agente - SIMPLIFICADO *)
fair process P2PAgent = "agent"
variables cycles = 0;
begin
    AgentLoop:
        while cycles < 3 do
            (* IDLE *)
            IdleState:
                status := "Idle";

            
            (* FETCHING *)
            StartFetch:
                status := "FetchingPeers";
                last_fetch_time := clock;
            
            CallRPC:
                either
                    (* Sucesso - recebe peers *)
                    RpcSuccess:
                        num_peers := 2;  \* Simplificado: sempre 2 peers
                or
                    (* Falha - sem peers *)
                    RpcFailure:
                        num_peers := 0;
                end either;
            
            (* PROBING - Só se tem peers *)
            CheckProbing:
                if num_peers > 0 then
                    ProbeState:
                        status := "Probing";
                    
                    DoProbe:
                        either
                            probe_success := TRUE;   \* Sucesso
                        or
                            probe_success := FALSE;  \* Falha
                        end either;
                    
                    FinishProbe:
                        status := "Reporting";
                else
                    SkipProbe:
                        skip;  \* Pula probing, volta ao idle
                end if;
            
            (* REPORTING - Só se sondou *)
            CheckReport:
                if status = "Reporting" then
                    SendReport:
                        skip;  \* Simula envio
                    
                    BackToIdle:
                        status := "Idle";
                end if;
            IncrementCycle:
                cycles := cycles + 1;
        end while;
end process;

end algorithm; *)

\* BEGIN TRANSLATION (chksum(pcal) = "353c0122" /\ chksum(tla) = "846400f3")
VARIABLES status, num_peers, probe_success, clock, last_fetch_time, pc

(* define statement *)
TypeInvariant ==
    /\ status \in {"Idle", "FetchingPeers", "Probing", "Reporting"}
    /\ clock >= 0
    /\ last_fetch_time >= -10
    /\ num_peers >= 0


EventuallyIdle == <>(status = "Idle")


EventuallyProbe == (num_peers > 0) ~> (status = "Probing")

VARIABLE cycles

vars == << status, num_peers, probe_success, clock, last_fetch_time, pc, 
           cycles >>

ProcSet == {"clock"} \cup {"agent"}

Init == (* Global variables *)
        /\ status = "Idle"
        /\ num_peers = 0
        /\ probe_success = FALSE
        /\ clock = 0
        /\ last_fetch_time = -10
        (* Process P2PAgent *)
        /\ cycles = 0
        /\ pc = [self \in ProcSet |-> CASE self = "clock" -> "ClockLoop"
                                        [] self = "agent" -> "AgentLoop"]

ClockLoop == /\ pc["clock"] = "ClockLoop"
             /\ IF clock < MAX_CLOCK
                   THEN /\ pc' = [pc EXCEPT !["clock"] = "Tick"]
                   ELSE /\ pc' = [pc EXCEPT !["clock"] = "Done"]
             /\ UNCHANGED << status, num_peers, probe_success, clock, 
                             last_fetch_time, cycles >>

Tick == /\ pc["clock"] = "Tick"
        /\ clock' = clock + 1
        /\ pc' = [pc EXCEPT !["clock"] = "ClockLoop"]
        /\ UNCHANGED << status, num_peers, probe_success, last_fetch_time, 
                        cycles >>

Clock == ClockLoop \/ Tick

AgentLoop == /\ pc["agent"] = "AgentLoop"
             /\ IF cycles < 3
                   THEN /\ pc' = [pc EXCEPT !["agent"] = "IdleState"]
                   ELSE /\ pc' = [pc EXCEPT !["agent"] = "Done"]
             /\ UNCHANGED << status, num_peers, probe_success, clock, 
                             last_fetch_time, cycles >>

IdleState == /\ pc["agent"] = "IdleState"
             /\ status' = "Idle"
             /\ pc' = [pc EXCEPT !["agent"] = "StartFetch"]
             /\ UNCHANGED << num_peers, probe_success, clock, last_fetch_time, 
                             cycles >>

StartFetch == /\ pc["agent"] = "StartFetch"
              /\ status' = "FetchingPeers"
              /\ last_fetch_time' = clock
              /\ pc' = [pc EXCEPT !["agent"] = "CallRPC"]
              /\ UNCHANGED << num_peers, probe_success, clock, cycles >>

CallRPC == /\ pc["agent"] = "CallRPC"
           /\ \/ /\ pc' = [pc EXCEPT !["agent"] = "RpcSuccess"]
              \/ /\ pc' = [pc EXCEPT !["agent"] = "RpcFailure"]
           /\ UNCHANGED << status, num_peers, probe_success, clock, 
                           last_fetch_time, cycles >>

RpcSuccess == /\ pc["agent"] = "RpcSuccess"
              /\ num_peers' = 2
              /\ pc' = [pc EXCEPT !["agent"] = "CheckProbing"]
              /\ UNCHANGED << status, probe_success, clock, last_fetch_time, 
                              cycles >>

RpcFailure == /\ pc["agent"] = "RpcFailure"
              /\ num_peers' = 0
              /\ pc' = [pc EXCEPT !["agent"] = "CheckProbing"]
              /\ UNCHANGED << status, probe_success, clock, last_fetch_time, 
                              cycles >>

CheckProbing == /\ pc["agent"] = "CheckProbing"
                /\ IF num_peers > 0
                      THEN /\ pc' = [pc EXCEPT !["agent"] = "ProbeState"]
                      ELSE /\ pc' = [pc EXCEPT !["agent"] = "SkipProbe"]
                /\ UNCHANGED << status, num_peers, probe_success, clock, 
                                last_fetch_time, cycles >>

ProbeState == /\ pc["agent"] = "ProbeState"
              /\ status' = "Probing"
              /\ pc' = [pc EXCEPT !["agent"] = "DoProbe"]
              /\ UNCHANGED << num_peers, probe_success, clock, last_fetch_time, 
                              cycles >>

DoProbe == /\ pc["agent"] = "DoProbe"
           /\ \/ /\ probe_success' = TRUE
              \/ /\ probe_success' = FALSE
           /\ pc' = [pc EXCEPT !["agent"] = "FinishProbe"]
           /\ UNCHANGED << status, num_peers, clock, last_fetch_time, cycles >>

FinishProbe == /\ pc["agent"] = "FinishProbe"
               /\ status' = "Reporting"
               /\ pc' = [pc EXCEPT !["agent"] = "CheckReport"]
               /\ UNCHANGED << num_peers, probe_success, clock, 
                               last_fetch_time, cycles >>

SkipProbe == /\ pc["agent"] = "SkipProbe"
             /\ TRUE
             /\ pc' = [pc EXCEPT !["agent"] = "CheckReport"]
             /\ UNCHANGED << status, num_peers, probe_success, clock, 
                             last_fetch_time, cycles >>

CheckReport == /\ pc["agent"] = "CheckReport"
               /\ IF status = "Reporting"
                     THEN /\ pc' = [pc EXCEPT !["agent"] = "SendReport"]
                     ELSE /\ pc' = [pc EXCEPT !["agent"] = "IncrementCycle"]
               /\ UNCHANGED << status, num_peers, probe_success, clock, 
                               last_fetch_time, cycles >>

SendReport == /\ pc["agent"] = "SendReport"
              /\ TRUE
              /\ pc' = [pc EXCEPT !["agent"] = "BackToIdle"]
              /\ UNCHANGED << status, num_peers, probe_success, clock, 
                              last_fetch_time, cycles >>

BackToIdle == /\ pc["agent"] = "BackToIdle"
              /\ status' = "Idle"
              /\ pc' = [pc EXCEPT !["agent"] = "IncrementCycle"]
              /\ UNCHANGED << num_peers, probe_success, clock, last_fetch_time, 
                              cycles >>

IncrementCycle == /\ pc["agent"] = "IncrementCycle"
                  /\ cycles' = cycles + 1
                  /\ pc' = [pc EXCEPT !["agent"] = "AgentLoop"]
                  /\ UNCHANGED << status, num_peers, probe_success, clock, 
                                  last_fetch_time >>

P2PAgent == AgentLoop \/ IdleState \/ StartFetch \/ CallRPC \/ RpcSuccess
               \/ RpcFailure \/ CheckProbing \/ ProbeState \/ DoProbe
               \/ FinishProbe \/ SkipProbe \/ CheckReport \/ SendReport
               \/ BackToIdle \/ IncrementCycle

(* Allow infinite stuttering to prevent deadlock on termination. *)
Terminating == /\ \A self \in ProcSet: pc[self] = "Done"
               /\ UNCHANGED vars

Next == Clock \/ P2PAgent
           \/ Terminating

Spec == /\ Init /\ [][Next]_vars
        /\ WF_vars(Clock)
        /\ WF_vars(P2PAgent)

Termination == <>(\A self \in ProcSet: pc[self] = "Done")

\* END TRANSLATION

====
