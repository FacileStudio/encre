// Package encre runs the encre command-line tool as a subprocess, so Go services convert, validate and
// render encre documents with the same Rust code as every other caller.
//
// The binary is ENCRE_BIN when set, else "encre" on PATH.
package encre

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"time"
)

// DefaultTimeout bounds a call whose context has no deadline. Give a long PDF its own deadline.
const DefaultTimeout = 30 * time.Second

// Error is a failure encre reported, with its exit code.
type Error struct {
	Exit    int    `json:"exit"`
	Code    string `json:"code"`
	Message string `json:"message"`
	Path    string `json:"path"`
}

// Error formats the failure with its exit code.
func (e *Error) Error() string { return fmt.Sprintf("encre: %s (exit %d)", e.Message, e.Exit) }

// Invalid reports whether the input was at fault, so the caller answers 4xx, not 5xx.
func (e *Error) Invalid() bool { return e.Exit == 1 }

// Binary is the encre executable a call runs.
func Binary() string {
	if bin := os.Getenv("ENCRE_BIN"); bin != "" {
		return bin
	}
	return "encre"
}

type result struct {
	stdout, stderr []byte
	exit           int
}

func exec1(ctx context.Context, stdin []byte, args []string) (result, error) {
	if _, ok := ctx.Deadline(); !ok {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(ctx, DefaultTimeout)
		defer cancel()
	}
	cmd := exec.CommandContext(ctx, Binary(), append(args, "--json")...)
	cmd.Stdin = bytes.NewReader(stdin)
	var out, errOut bytes.Buffer
	cmd.Stdout, cmd.Stderr = &out, &errOut
	err := cmd.Run()
	if ctx.Err() != nil {
		return result{}, fmt.Errorf("encre %s: %w", args[0], ctx.Err())
	}
	var exitErr *exec.ExitError
	if errors.As(err, &exitErr) {
		return result{out.Bytes(), errOut.Bytes(), exitErr.ExitCode()}, nil
	}
	if err != nil {
		return result{}, fmt.Errorf("encre %s: %w", args[0], err)
	}
	return result{out.Bytes(), errOut.Bytes(), 0}, nil
}

func refusal(command string, stderr []byte, exit int) error {
	var body struct {
		Error Error `json:"error"`
	}
	if json.Unmarshal(stderr, &body) != nil {
		return fmt.Errorf("encre %s: exit %d: %s", command, exit, bytes.TrimSpace(stderr))
	}
	body.Error.Exit = exit
	return &body.Error
}

// Run runs encre with args and --json, feeds it stdin, and returns stdout. A refusal is an *Error.
func Run(ctx context.Context, stdin []byte, args ...string) ([]byte, error) {
	if len(args) == 0 {
		return nil, errors.New("encre: no command")
	}
	res, err := exec1(ctx, stdin, args)
	if err != nil {
		return nil, err
	}
	if res.exit != 0 {
		return nil, refusal(args[0], res.stderr, res.exit)
	}
	return res.stdout, nil
}
