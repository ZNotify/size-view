package internal

import (
	"bytes"
	"github.com/stretchr/testify/require"
	"testing"
)

func FuzzAnalyze(f *testing.F) {
	f.Fuzz(func(t *testing.T, name string, data []byte) {
		require.NotPanics(t, func() {
			reader := bytes.NewReader(data)
			_, err := Analyze(name, reader, uint64(len(data)), Options{})
			if err != nil {
				t.Logf("Error: %v", err)
			}
		})
	})
}
