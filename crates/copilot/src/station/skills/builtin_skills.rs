use super::skill_types::{SkillDefinition, SkillStep};

/// Return all 16 built-in skill definitions.
///
/// Skills are divided into two groups:
/// - **Carried from v2.x** (6): core orchestration patterns
/// - **New for v3.0** (10): domain-specific workflows
pub fn all_skills() -> Vec<SkillDefinition> {
    vec![
        // Carried from v2.x (6)
        structured_planning(),
        diagnostic_reasoning(),
        self_verification(),
        parallel_dispatch(),
        context_accumulation(),
        task_handoff(),
        // New for v3.0 (10)
        deep_research(),
        comparative_analysis(),
        report_writing(),
        email_drafting(),
        data_summarization(),
        code_review(),
        web_automation(),
        meeting_prep(),
        content_repurpose(),
        fact_checking(),
    ]
}

// ---------------------------------------------------------------------------
// Carried from v2.x (6 skills)
// ---------------------------------------------------------------------------

fn structured_planning() -> SkillDefinition {
    SkillDefinition {
        id: "structured-planning".into(),
        name: "Structured Planning".into(),
        description: "Break a high-level goal into an ordered, dependency-aware task plan".into(),
        used_by: vec!["orchestrator".into()],
        steps: vec![
            SkillStep {
                title: "Goal Analysis".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "Analyze the user's goal: {{goal}}. Identify the key \
                    deliverables, constraints, and success criteria."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Task Decomposition".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "Decompose the goal into discrete subtasks based on the \
                    analysis: {{analysis}}. For each subtask specify: title, description, \
                    suggested agent role, estimated effort, and dependencies."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Dependency Ordering".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "Given the subtasks: {{subtasks}}, determine execution order. \
                    Identify which tasks can run in parallel and which have serial dependencies. \
                    Output a DAG-style execution plan."
                    .into(),
                tools: vec!["memory".into()],
            },
        ],
    }
}

fn diagnostic_reasoning() -> SkillDefinition {
    SkillDefinition {
        id: "diagnostic-reasoning".into(),
        name: "Diagnostic Reasoning".into(),
        description: "Systematically diagnose problems using hypothesis-test cycles".into(),
        used_by: vec!["orchestrator".into(), "analyst".into()],
        steps: vec![
            SkillStep {
                title: "Symptom Collection".into(),
                agent_role: "analyst".into(),
                prompt_template: "Collect and list all observed symptoms for the problem: \
                    {{problem}}. Include error messages, unexpected behaviors, and environmental \
                    factors."
                    .into(),
                tools: vec!["memory".into(), "web-search".into()],
            },
            SkillStep {
                title: "Hypothesis Generation".into(),
                agent_role: "analyst".into(),
                prompt_template: "Based on symptoms: {{symptoms}}, generate ranked hypotheses \
                    for root cause. For each hypothesis state the expected evidence that would \
                    confirm or refute it."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Test & Conclude".into(),
                agent_role: "analyst".into(),
                prompt_template: "Test the top hypotheses: {{hypotheses}}. For each, describe \
                    the test performed, outcome observed, and whether it was confirmed or \
                    refuted. Conclude with the most likely root cause and recommended fix."
                    .into(),
                tools: vec!["memory".into(), "web-search".into()],
            },
        ],
    }
}

fn self_verification() -> SkillDefinition {
    SkillDefinition {
        id: "self-verification".into(),
        name: "Self-Verification".into(),
        description: "Verify an agent's own output for correctness and completeness".into(),
        used_by: vec!["orchestrator".into(), "coder".into(), "writer".into()],
        steps: vec![
            SkillStep {
                title: "Output Review".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "Review the output produced by {{agent}}: {{output}}. \
                    Check for factual accuracy, internal consistency, and completeness \
                    against the original requirements: {{requirements}}."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Error Detection".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "Identify specific errors, omissions, or quality issues in \
                    the reviewed output: {{review}}. Categorize each issue as critical, major, \
                    or minor."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Correction & Sign-off".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "Apply corrections for all critical and major issues found: \
                    {{issues}}. Produce the final corrected output and a brief verification \
                    summary."
                    .into(),
                tools: vec!["memory".into()],
            },
        ],
    }
}

fn parallel_dispatch() -> SkillDefinition {
    SkillDefinition {
        id: "parallel-dispatch".into(),
        name: "Parallel Dispatch".into(),
        description: "Fan out independent subtasks to multiple agents simultaneously".into(),
        used_by: vec!["orchestrator".into()],
        steps: vec![
            SkillStep {
                title: "Identify Parallelizable Work".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "From the task plan: {{plan}}, identify groups of subtasks \
                    that have no mutual dependencies and can execute in parallel."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Dispatch to Agents".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "Dispatch parallel groups: {{groups}}. Assign each subtask \
                    to the appropriate agent based on role. Track dispatch timestamps and \
                    expected completion."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Collect & Merge Results".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "Collect results from parallel agents: {{results}}. Merge \
                    outputs, resolve any conflicts between overlapping work, and prepare the \
                    combined result for the next serial stage."
                    .into(),
                tools: vec!["memory".into()],
            },
        ],
    }
}

fn context_accumulation() -> SkillDefinition {
    SkillDefinition {
        id: "context-accumulation".into(),
        name: "Context Accumulation".into(),
        description: "Progressively gather and compress context across long-running tasks".into(),
        used_by: vec!["orchestrator".into(), "researcher".into()],
        steps: vec![
            SkillStep {
                title: "Gather Fragments".into(),
                agent_role: "researcher".into(),
                prompt_template: "Gather all relevant context fragments for the ongoing task: \
                    {{task}}. Include prior step outputs, user messages, and external data."
                    .into(),
                tools: vec!["memory".into(), "web-fetch".into()],
            },
            SkillStep {
                title: "Compress & Prioritize".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "Compress the gathered context: {{fragments}}. Remove \
                    redundancy, rank information by relevance, and produce a concise context \
                    summary that fits within the token budget."
                    .into(),
                tools: vec!["memory".into()],
            },
        ],
    }
}

fn task_handoff() -> SkillDefinition {
    SkillDefinition {
        id: "task-handoff".into(),
        name: "Task Handoff".into(),
        description: "Transfer work between agents with a structured handoff brief".into(),
        used_by: vec!["orchestrator".into()],
        steps: vec![
            SkillStep {
                title: "Prepare Handoff Brief".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "Prepare a handoff brief from {{source_agent}} to \
                    {{target_agent}} for task: {{task}}. Include: completed work summary, \
                    key findings, open questions, and next steps."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Validate & Transfer".into(),
                agent_role: "orchestrator".into(),
                prompt_template: "Validate the handoff brief: {{brief}}. Ensure all required \
                    artifacts are attached and the receiving agent has sufficient context. \
                    Execute the transfer."
                    .into(),
                tools: vec!["memory".into()],
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// New for v3.0 (10 skills)
// ---------------------------------------------------------------------------

fn deep_research() -> SkillDefinition {
    SkillDefinition {
        id: "deep-research".into(),
        name: "Deep Research".into(),
        description: "Multi-hop web search with cross-reference and confidence scoring".into(),
        used_by: vec!["researcher".into()],
        steps: vec![
            SkillStep {
                title: "Initial Search".into(),
                agent_role: "researcher".into(),
                prompt_template: "Search the web for information about: {{topic}}. Find at \
                    least 5 diverse sources. Return structured results with URL, title, and \
                    key findings."
                    .into(),
                tools: vec!["web-search".into(), "web-fetch".into()],
            },
            SkillStep {
                title: "Cross-Reference".into(),
                agent_role: "researcher".into(),
                prompt_template: "Cross-reference the findings from these sources: {{sources}}. \
                    Identify agreements, contradictions, and gaps. Rate confidence \
                    (high/medium/low) for each claim."
                    .into(),
                tools: vec!["web-fetch".into(), "memory".into()],
            },
            SkillStep {
                title: "Extract & Score".into(),
                agent_role: "analyst".into(),
                prompt_template: "From the cross-referenced research: {{findings}}. Extract \
                    the top facts, assign confidence scores, and flag unverifiable claims."
                    .into(),
                tools: vec!["memory".into()],
            },
        ],
    }
}

fn comparative_analysis() -> SkillDefinition {
    SkillDefinition {
        id: "comparative-analysis".into(),
        name: "Comparative Analysis".into(),
        description: "Side-by-side evaluation of options with weighted scoring".into(),
        used_by: vec!["analyst".into()],
        steps: vec![
            SkillStep {
                title: "Gather Options".into(),
                agent_role: "researcher".into(),
                prompt_template: "Identify and describe the options to compare for: \
                    {{comparison_topic}}. For each option collect key features, pricing, \
                    pros, and cons."
                    .into(),
                tools: vec!["web-search".into(), "web-fetch".into()],
            },
            SkillStep {
                title: "Define Criteria".into(),
                agent_role: "analyst".into(),
                prompt_template: "Define evaluation criteria for comparing: {{options}}. \
                    Assign a weight (1-5) to each criterion based on the user's stated \
                    priorities: {{priorities}}."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Score & Rank".into(),
                agent_role: "analyst".into(),
                prompt_template: "Score each option against the weighted criteria: \
                    {{criteria}}. Produce a ranked comparison table with total scores and \
                    a recommendation with rationale."
                    .into(),
                tools: vec!["memory".into()],
            },
        ],
    }
}

fn report_writing() -> SkillDefinition {
    SkillDefinition {
        id: "report-writing".into(),
        name: "Report Writing".into(),
        description: "Structured report generation with outline, draft, and polish phases".into(),
        used_by: vec!["writer".into()],
        steps: vec![
            SkillStep {
                title: "Outline".into(),
                agent_role: "writer".into(),
                prompt_template: "Create a detailed outline for a report on: {{topic}}. \
                    Include section headings, key points per section, and estimated word \
                    counts. Target audience: {{audience}}."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Draft".into(),
                agent_role: "writer".into(),
                prompt_template: "Write a complete first draft following the outline: \
                    {{outline}}. Use the research data: {{research}}. Maintain a \
                    {{tone}} tone throughout."
                    .into(),
                tools: vec!["memory".into(), "document".into()],
            },
            SkillStep {
                title: "Polish & Proofread".into(),
                agent_role: "writer".into(),
                prompt_template: "Polish the draft: {{draft}}. Fix grammar, improve \
                    readability, ensure logical flow, and verify all citations. Add an \
                    executive summary at the top."
                    .into(),
                tools: vec!["memory".into(), "document".into()],
            },
        ],
    }
}

fn email_drafting() -> SkillDefinition {
    SkillDefinition {
        id: "email-drafting".into(),
        name: "Email Drafting".into(),
        description: "Context-aware email composition with tone matching and review".into(),
        used_by: vec!["writer".into()],
        steps: vec![
            SkillStep {
                title: "Context & Intent".into(),
                agent_role: "writer".into(),
                prompt_template: "Understand the email context: recipient={{recipient}}, \
                    purpose={{purpose}}, prior thread={{thread}}. Determine the appropriate \
                    tone, formality level, and key points to address."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Draft Email".into(),
                agent_role: "writer".into(),
                prompt_template: "Draft the email based on: {{context}}. Include a clear \
                    subject line, greeting, body with the key points, and a call to action. \
                    Keep it concise."
                    .into(),
                tools: vec!["memory".into(), "document".into()],
            },
            SkillStep {
                title: "Tone Review".into(),
                agent_role: "writer".into(),
                prompt_template: "Review the email draft: {{draft}}. Check tone consistency, \
                    clarity, and professionalism. Suggest edits if the tone doesn't match \
                    the target: {{tone}}. Produce the final version."
                    .into(),
                tools: vec!["memory".into()],
            },
        ],
    }
}

fn data_summarization() -> SkillDefinition {
    SkillDefinition {
        id: "data-summarization".into(),
        name: "Data Summarization".into(),
        description: "Extract insights and summaries from structured or unstructured data".into(),
        used_by: vec!["analyst".into()],
        steps: vec![
            SkillStep {
                title: "Ingest Data".into(),
                agent_role: "analyst".into(),
                prompt_template: "Ingest and parse the data: {{data_source}}. Identify the \
                    data format, schema, and total record count. Report any quality issues \
                    (missing values, outliers, inconsistencies)."
                    .into(),
                tools: vec!["data".into(), "memory".into()],
            },
            SkillStep {
                title: "Analyze Patterns".into(),
                agent_role: "analyst".into(),
                prompt_template: "Analyze the ingested data: {{data}}. Identify key trends, \
                    distributions, correlations, and outliers. Compute relevant statistics."
                    .into(),
                tools: vec!["data".into(), "memory".into()],
            },
            SkillStep {
                title: "Summarize & Visualize".into(),
                agent_role: "analyst".into(),
                prompt_template: "Produce a narrative summary of the analysis: {{analysis}}. \
                    Highlight the top 3-5 insights, provide supporting data points, and \
                    suggest chart types for visual presentation."
                    .into(),
                tools: vec!["memory".into(), "document".into()],
            },
        ],
    }
}

fn code_review() -> SkillDefinition {
    SkillDefinition {
        id: "code-review".into(),
        name: "Code Review".into(),
        description: "Automated code review covering style, bugs, security, and performance".into(),
        used_by: vec!["coder".into()],
        steps: vec![
            SkillStep {
                title: "Static Analysis".into(),
                agent_role: "coder".into(),
                prompt_template: "Perform static analysis on the code: {{code}}. Check for \
                    syntax errors, linting violations, naming conventions, and code style \
                    issues against {{style_guide}} standards."
                    .into(),
                tools: vec!["filesystem".into(), "memory".into()],
            },
            SkillStep {
                title: "Bug & Security Scan".into(),
                agent_role: "coder".into(),
                prompt_template: "Scan the code for potential bugs and security \
                    vulnerabilities: {{code}}. Check for: null pointer issues, injection \
                    flaws, hardcoded secrets, race conditions, and error handling gaps."
                    .into(),
                tools: vec!["filesystem".into(), "memory".into()],
            },
            SkillStep {
                title: "Performance & Summary".into(),
                agent_role: "coder".into(),
                prompt_template: "Review performance characteristics of: {{code}}. Identify \
                    potential bottlenecks, unnecessary allocations, and O(n) complexity \
                    concerns. Produce a review summary with severity-ranked findings."
                    .into(),
                tools: vec!["memory".into()],
            },
        ],
    }
}

fn web_automation() -> SkillDefinition {
    SkillDefinition {
        id: "web-automation".into(),
        name: "Web Automation".into(),
        description: "Browser-based task automation with page interaction and data extraction"
            .into(),
        used_by: vec!["operator".into()],
        steps: vec![
            SkillStep {
                title: "Plan Navigation".into(),
                agent_role: "operator".into(),
                prompt_template: "Plan the browser automation steps to accomplish: {{task}}. \
                    Identify the target URL, required form fields, login steps (if any), \
                    and expected page states."
                    .into(),
                tools: vec!["memory".into(), "web-search".into()],
            },
            SkillStep {
                title: "Execute Actions".into(),
                agent_role: "operator".into(),
                prompt_template: "Execute the planned browser steps: {{plan}}. Navigate to \
                    pages, fill forms, click buttons, and wait for page loads. Capture \
                    screenshots at key checkpoints."
                    .into(),
                tools: vec!["browser".into(), "memory".into()],
            },
            SkillStep {
                title: "Extract & Verify".into(),
                agent_role: "operator".into(),
                prompt_template: "Extract the target data from the automated session: \
                    {{session}}. Verify the extracted data matches expectations. Report \
                    any errors or incomplete steps."
                    .into(),
                tools: vec!["browser".into(), "memory".into(), "data".into()],
            },
        ],
    }
}

fn meeting_prep() -> SkillDefinition {
    SkillDefinition {
        id: "meeting-prep".into(),
        name: "Meeting Preparation".into(),
        description: "Prepare briefing materials, talking points, and agendas for meetings".into(),
        used_by: vec!["researcher".into(), "writer".into()],
        steps: vec![
            SkillStep {
                title: "Background Research".into(),
                agent_role: "researcher".into(),
                prompt_template: "Research background for the meeting with: {{participants}} \
                    about: {{topic}}. Gather recent news, company updates, and relevant \
                    context for each participant."
                    .into(),
                tools: vec!["web-search".into(), "web-fetch".into(), "memory".into()],
            },
            SkillStep {
                title: "Talking Points".into(),
                agent_role: "writer".into(),
                prompt_template: "Based on the research: {{research}}, create a structured \
                    meeting brief with: agenda items, talking points per item, potential \
                    questions to ask, and desired outcomes."
                    .into(),
                tools: vec!["memory".into(), "document".into()],
            },
            SkillStep {
                title: "One-Pager".into(),
                agent_role: "writer".into(),
                prompt_template: "Compile the talking points: {{talking_points}} into a \
                    concise one-page meeting prep document. Include a 2-sentence summary \
                    at the top, key discussion items in bullet form, and action items to \
                    propose."
                    .into(),
                tools: vec!["memory".into(), "document".into()],
            },
        ],
    }
}

fn content_repurpose() -> SkillDefinition {
    SkillDefinition {
        id: "content-repurpose".into(),
        name: "Content Repurposing".into(),
        description: "Transform content between formats: blog post, tweet thread, slides, etc."
            .into(),
        used_by: vec!["writer".into()],
        steps: vec![
            SkillStep {
                title: "Analyze Source".into(),
                agent_role: "writer".into(),
                prompt_template: "Analyze the source content: {{source}}. Identify the key \
                    messages, data points, quotes, and narrative structure. Note the original \
                    format: {{source_format}}."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Adapt to Target".into(),
                agent_role: "writer".into(),
                prompt_template: "Repurpose the analyzed content: {{analysis}} into the \
                    target format: {{target_format}}. Adapt tone, length, and structure \
                    to match the target medium's conventions and audience expectations."
                    .into(),
                tools: vec!["memory".into(), "document".into()],
            },
            SkillStep {
                title: "Quality Check".into(),
                agent_role: "writer".into(),
                prompt_template: "Review the repurposed content: {{repurposed}}. Verify it \
                    preserves the core message from the source while fitting the target \
                    format. Check for format-specific issues (character limits, heading \
                    structure, etc.)."
                    .into(),
                tools: vec!["memory".into()],
            },
        ],
    }
}

fn fact_checking() -> SkillDefinition {
    SkillDefinition {
        id: "fact-checking".into(),
        name: "Fact Checking".into(),
        description: "Verify claims against multiple sources with confidence verdicts".into(),
        used_by: vec!["researcher".into(), "analyst".into()],
        steps: vec![
            SkillStep {
                title: "Extract Claims".into(),
                agent_role: "analyst".into(),
                prompt_template: "Extract all factual claims from the content: {{content}}. \
                    List each claim separately with its source context. Categorize as: \
                    statistical, historical, scientific, or opinion."
                    .into(),
                tools: vec!["memory".into()],
            },
            SkillStep {
                title: "Source Verification".into(),
                agent_role: "researcher".into(),
                prompt_template: "For each extracted claim: {{claims}}, search for \
                    corroborating or contradicting evidence from authoritative sources. \
                    Find at least 2 independent sources per claim."
                    .into(),
                tools: vec!["web-search".into(), "web-fetch".into(), "memory".into()],
            },
            SkillStep {
                title: "Verdict & Report".into(),
                agent_role: "analyst".into(),
                prompt_template: "Based on the verification results: {{verification}}, assign \
                    a verdict to each claim: Confirmed, Likely True, Unverifiable, Likely \
                    False, or False. Produce a fact-check report with sources cited."
                    .into(),
                tools: vec!["memory".into(), "document".into()],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_skills_count() {
        let skills = all_skills();
        assert_eq!(skills.len(), 16, "expected 16 built-in skills");
    }

    #[test]
    fn test_all_skills_have_unique_ids() {
        let skills = all_skills();
        let mut ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "skill IDs must be unique");
    }

    #[test]
    fn test_all_skills_have_at_least_one_step() {
        for skill in all_skills() {
            assert!(
                !skill.steps.is_empty(),
                "skill '{}' must have at least 1 step",
                skill.id
            );
        }
    }

    #[test]
    fn test_all_skills_have_names() {
        for skill in all_skills() {
            assert!(
                !skill.name.is_empty(),
                "skill '{}' must have a name",
                skill.id
            );
            assert!(
                !skill.description.is_empty(),
                "skill '{}' must have a description",
                skill.id
            );
        }
    }

    #[test]
    fn test_all_skills_have_used_by() {
        for skill in all_skills() {
            assert!(
                !skill.used_by.is_empty(),
                "skill '{}' must specify at least one agent in used_by",
                skill.id
            );
        }
    }

    #[test]
    fn test_all_steps_have_tools() {
        for skill in all_skills() {
            for step in &skill.steps {
                assert!(
                    !step.tools.is_empty(),
                    "skill '{}' step '{}' must list at least 1 tool",
                    skill.id,
                    step.title
                );
            }
        }
    }

    #[test]
    fn test_all_steps_have_templates() {
        for skill in all_skills() {
            for step in &skill.steps {
                assert!(
                    !step.prompt_template.is_empty(),
                    "skill '{}' step '{}' must have a prompt_template",
                    skill.id,
                    step.title
                );
                assert!(
                    step.prompt_template.contains("{{"),
                    "skill '{}' step '{}' prompt_template should contain placeholders",
                    skill.id,
                    step.title
                );
            }
        }
    }

    #[test]
    fn test_deep_research_skill() {
        let skill = deep_research();
        assert_eq!(skill.id, "deep-research");
        assert_eq!(skill.steps.len(), 3);
        assert_eq!(skill.used_by, vec!["researcher"]);

        assert_eq!(skill.steps[0].title, "Initial Search");
        assert_eq!(skill.steps[0].agent_role, "researcher");
        assert!(skill.steps[0].tools.contains(&"web-search".to_string()));

        assert_eq!(skill.steps[1].title, "Cross-Reference");
        assert_eq!(skill.steps[2].title, "Extract & Score");
        assert_eq!(skill.steps[2].agent_role, "analyst");
    }

    #[test]
    fn test_code_review_skill() {
        let skill = code_review();
        assert_eq!(skill.id, "code-review");
        assert_eq!(skill.steps.len(), 3);
        assert_eq!(skill.used_by, vec!["coder"]);
        assert_eq!(skill.steps[0].agent_role, "coder");
    }

    #[test]
    fn test_v2_skills_present() {
        let skills = all_skills();
        let v2_ids = [
            "structured-planning",
            "diagnostic-reasoning",
            "self-verification",
            "parallel-dispatch",
            "context-accumulation",
            "task-handoff",
        ];
        for expected_id in &v2_ids {
            assert!(
                skills.iter().any(|s| s.id == *expected_id),
                "v2 skill '{}' must be present",
                expected_id
            );
        }
    }

    #[test]
    fn test_v3_skills_present() {
        let skills = all_skills();
        let v3_ids = [
            "deep-research",
            "comparative-analysis",
            "report-writing",
            "email-drafting",
            "data-summarization",
            "code-review",
            "web-automation",
            "meeting-prep",
            "content-repurpose",
            "fact-checking",
        ];
        for expected_id in &v3_ids {
            assert!(
                skills.iter().any(|s| s.id == *expected_id),
                "v3 skill '{}' must be present",
                expected_id
            );
        }
    }

    #[test]
    fn test_agent_roles_are_valid() {
        let valid_roles = [
            "orchestrator",
            "researcher",
            "analyst",
            "writer",
            "coder",
            "operator",
        ];
        for skill in all_skills() {
            for agent in &skill.used_by {
                assert!(
                    valid_roles.contains(&agent.as_str()),
                    "skill '{}' references unknown agent role '{}'",
                    skill.id,
                    agent
                );
            }
            for step in &skill.steps {
                assert!(
                    valid_roles.contains(&step.agent_role.as_str()),
                    "skill '{}' step '{}' references unknown agent_role '{}'",
                    skill.id,
                    step.title,
                    step.agent_role
                );
            }
        }
    }
}
