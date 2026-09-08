// SPDX-License-Identifier: Apache-2.0

package e2esuite

import (
	"fmt"
	"sync"
)

// ParallelTask represents a named task that can be executed concurrently
type ParallelTask struct {
	Name string
	Run  func() error
}

// RunParallelTasks executes multiple tasks concurrently and collects their results.
// Returns an error if any task fails, with the task name included in the error message.
//
// Example usage:
//
//	err := e2esuite.RunParallelTasks(
//	    e2esuite.ParallelTask{
//	        Name: "Setup Solana",
//	        Run: func() error {
//	            // ... setup logic
//	            return nil
//	        },
//	    },
//	    e2esuite.ParallelTask{
//	        Name: "Setup Cosmos",
//	        Run: func() error {
//	            // ... setup logic
//	            return nil
//	        },
//	    },
//	)
//	if err != nil {
//	    return err
//	}
func RunParallelTasks(tasks ...ParallelTask) error {
	if len(tasks) == 0 {
		return nil
	}

	type result struct {
		name string
		err  error
	}

	results := make(chan result, len(tasks))

	// Execute all tasks concurrently
	for _, task := range tasks {
		task := task // capture loop variable
		go func() {
			err := task.Run()
			results <- result{name: task.Name, err: err}
		}()
	}

	// Collect results and check for errors
	var firstError error
	for i := 0; i < len(tasks); i++ {
		res := <-results
		if res.err != nil && firstError == nil {
			firstError = fmt.Errorf("%s failed: %w", res.name, res.err)
		}
	}

	return firstError
}

// ParallelTaskWithResult represents a named task that returns a value
type ParallelTaskWithResult[T any] struct {
	Name string
	Run  func() (T, error)
}

// RunParallelTasksWithResults executes multiple tasks concurrently and collects their results.
// Returns a map of task names to their results, or an error if any task fails.
//
// Example usage:
//
//	results, err := e2esuite.RunParallelTasksWithResults(
//	    e2esuite.ParallelTaskWithResult[[]ibc.Chain]{
//	        Name: "Build Cosmos chains",
//	        Run: func() ([]ibc.Chain, error) {
//	            // ... build chains
//	            return chains, nil
//	        },
//	    },
//	)
//	if err != nil {
//	    return err
//	}
//	chains := results["Build Cosmos chains"]
func RunParallelTasksWithResults[T any](tasks ...ParallelTaskWithResult[T]) (map[string]T, error) {
	if len(tasks) == 0 {
		return make(map[string]T), nil
	}

	type result struct {
		name  string
		value T
		err   error
	}

	results := make(chan result, len(tasks))

	// Execute all tasks concurrently
	for _, task := range tasks {
		task := task // capture loop variable
		go func() {
			value, err := task.Run()
			results <- result{name: task.Name, value: value, err: err}
		}()
	}

	// Collect results
	resultMap := make(map[string]T)
	var firstError error

	for i := 0; i < len(tasks); i++ {
		res := <-results
		if res.err != nil && firstError == nil {
			firstError = fmt.Errorf("%s failed: %w", res.name, res.err)
		} else if res.err == nil {
			resultMap[res.name] = res.value
		}
	}

	if firstError != nil {
		return nil, firstError
	}

	return resultMap, nil
}

// ParallelExecutor provides a fluent interface for building and executing parallel tasks
type ParallelExecutor struct {
	tasks []ParallelTask
	mu    sync.Mutex
}

// NewParallelExecutor creates a new parallel task executor
func NewParallelExecutor() *ParallelExecutor {
	return &ParallelExecutor{
		tasks: make([]ParallelTask, 0),
	}
}

// Add adds a new task to be executed in parallel
func (pe *ParallelExecutor) Add(name string, fn func() error) *ParallelExecutor {
	pe.mu.Lock()
	defer pe.mu.Unlock()
	pe.tasks = append(pe.tasks, ParallelTask{Name: name, Run: fn})
	return pe
}

// Run executes all added tasks in parallel and waits for completion
func (pe *ParallelExecutor) Run() error {
	return RunParallelTasks(pe.tasks...)
}
