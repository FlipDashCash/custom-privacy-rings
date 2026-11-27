package main

import (
	"fmt"
	"os"
	"strings"

	_ "github.com/helius-labs/zolana-custom-rings/prover/ring"
	"github.com/helius-labs/zolana/prover/server/app"
	"github.com/helius-labs/zolana/prover/server/logging"
	"github.com/rs/zerolog"
)

func main() {
	command := subcommand(os.Args[1:])
	// Proof errors can contain private witness values.
	if command == "start" || command == "prove" {
		zerolog.SetGlobalLevel(zerolog.Disabled)
	}
	if err := app.Run(os.Args); err != nil {
		switch command {
		case "start":
			// Proofs fail inside request handlers and start returns none of their errors.
			fmt.Fprintf(os.Stderr, "prover start failed: %v\n", err)
		case "prove":
			fmt.Fprintln(os.Stderr, "prover prove failed. Check the proving keys and backend configuration.")
		default:
			logging.Logger().Fatal().Err(err).Msg("App failed.")
		}
		os.Exit(1)
	}
}

func subcommand(args []string) string {
	// The prover accepts only boolean flags before the command.
	for _, arg := range args {
		if !strings.HasPrefix(arg, "-") {
			return arg
		}
	}
	return ""
}
