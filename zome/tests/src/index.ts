import {
  Config,
  NetworkType,
  InstallAgentsHapps,
  TransportConfigType,
} from "@holochain/tryorama";
import path from "path";
import messaging from "./messaging";
import receipts from "./receipts";
import signals from "./signals";
import pin from "./pin";
import jumps from "./jumps";
import playground from "./playground";

import { Installables } from "./types";

// QUIC
const network = {
  network_type: NetworkType.QuicBootstrap,
  transport_pool: [{ type: TransportConfigType.Quic }],
  bootstrap_service: "https://bootstrap-staging.holo.host/",
};

const conductorConfig = Config.gen({ network });

const p2pmessagedna = path.join(
  __dirname,
  "../../p2pmessage.workdir.dna/p2pmessage.dna"
);
const installAgent: InstallAgentsHapps = [[[p2pmessagedna]]];

const installables: Installables = {
  one: installAgent,
};

// messaging(conductorConfig, installables);
// receipts(conductorConfig, installables);
// signals(conductorConfig, installables);
// pin(conductorConfig, installables);
// jumps(conductorConfig, installables);
playground(conductorConfig, installables);
