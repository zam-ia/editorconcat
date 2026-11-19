// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Non-destructive compound clips backed by ordinary internal timelines.

use super::*;
use serde_json::Value;

const COMPOUND_ID: &str = "compoundTimelineId";
const COMPOUND_PARENT: &str = "compoundParentTimelineId";
const PARENT_TRACK: &str = "compoundParentTrackId";

pub(super) fn apply(
    project: &mut Project,
    mint: &mut IdMint,
    command: Command,
) -> Result<Outcome, CommandError> {
    match command {
        Command::CreateCompoundClip { clip_ids, name } => {
            let selected: HashSet<&str> = clip_ids.iter().map(String::as_str).collect();
            if selected.len() < 2 || selected.len() != clip_ids.len() {
                return Err(CommandError::CannotCompound);
            }
            let parent_id = project.active_timeline_id.clone();
            let parent = project.active();
            let mut clips: Vec<Clip> = clip_ids
                .iter()
                .map(|id| parent.clip(id).cloned().ok_or(CommandError::CannotCompound))
                .collect::<Result<_, _>>()?;
            if clips.iter().any(Clip::is_compound) {
                // Nested compounds need a cycle-aware graph. Refuse them
                // until that contract is exposed instead of risking recursion.
                return Err(CommandError::CannotCompound);
            }
            clips.sort_by(|a, b| a.start.total_cmp(&b.start));
            let start = clips.first().map_or(0.0, |clip| clip.start);
            let end = clips
                .iter()
                .map(|clip| clip.start + clip.duration)
                .fold(start, f64::max);
            let anchor = clips
                .iter()
                .find(|clip| clip.kind != ClipKind::Audio)
                .unwrap_or(&clips[0]);
            let anchor_track = anchor.track_id.clone();
            let video = parent.video;

            let internal_id = mint.next("tl");
            let mut track_map = std::collections::HashMap::new();
            let mut tracks = Vec::new();
            for parent_track in parent
                .tracks
                .iter()
                .filter(|track| clips.iter().any(|clip| clip.track_id == track.id))
            {
                let mut internal = parent_track.clone();
                let id = mint.next("t");
                internal.extra.insert(
                    PARENT_TRACK.to_owned(),
                    Value::String(parent_track.id.clone()),
                );
                track_map.insert(parent_track.id.clone(), id.clone());
                internal.id = id;
                tracks.push(internal);
            }
            for clip in &mut clips {
                clip.start -= start;
                clip.track_id = track_map
                    .get(&clip.track_id)
                    .cloned()
                    .ok_or(CommandError::TrackGone)?;
            }
            let mut extra = serde_json::Map::new();
            extra.insert("compound".to_owned(), Value::Bool(true));
            extra.insert(COMPOUND_PARENT.to_owned(), Value::String(parent_id));
            let internal = Timeline {
                id: internal_id.clone(),
                name: if name.trim().is_empty() {
                    "Clip compuesto".to_owned()
                } else {
                    name.trim().to_owned()
                },
                video,
                background: Background::None,
                tracks,
                clips: clips.into_iter().map(Arc::new).collect(),
                extra,
            };

            let id = mint.next("c");
            let mut compound = Clip::blank(
                id.clone(),
                anchor_track,
                ClipKind::Video,
                internal.name.clone(),
                start,
                end - start,
            );
            compound
                .extra
                .insert(COMPOUND_ID.to_owned(), Value::String(internal_id));

            let parent = project.active_mut();
            parent
                .clips
                .retain(|clip| !selected.contains(clip.id.as_str()));
            parent.clips.push(Arc::new(compound));
            project.timelines.push(Arc::new(internal));
            Ok(Outcome {
                created_id: Some(id),
                applied: true,
            })
        }

        Command::DecomposeCompoundClip { clip_id } => {
            let parent_index = project
                .timelines
                .iter()
                .position(|timeline| timeline.id == project.active_timeline_id)
                .ok_or(CommandError::NotCompound)?;
            let compound = project.timelines[parent_index]
                .clip(&clip_id)
                .cloned()
                .ok_or(CommandError::NotCompound)?;
            let internal_id = compound
                .compound_timeline_id()
                .ok_or(CommandError::NotCompound)?
                .to_owned();
            let internal_index = project
                .timelines
                .iter()
                .position(|timeline| timeline.id == internal_id)
                .ok_or(CommandError::NotCompound)?;
            let internal = project.timelines[internal_index].as_ref().clone();

            let parent_id = {
                let parent = Arc::make_mut(&mut project.timelines[parent_index]);
                let mut track_map = std::collections::HashMap::new();
                for track in &internal.tracks {
                    let previous = track
                        .extra
                        .get(PARENT_TRACK)
                        .and_then(Value::as_str)
                        .filter(|id| parent.track(id).is_some())
                        .map(str::to_owned);
                    let destination = previous.unwrap_or_else(|| {
                        let mut restored = track.clone();
                        restored.id = mint.next("t");
                        restored.extra.remove(PARENT_TRACK);
                        let id = restored.id.clone();
                        if restored.role == TrackRole::Audio {
                            parent.tracks.insert(0, restored);
                        } else {
                            parent.tracks.push(restored);
                        }
                        id
                    });
                    track_map.insert(track.id.clone(), destination);
                }
                for child in &internal.clips {
                    let mut child = Clip::clone(child);
                    child.track_id = track_map
                        .get(&child.track_id)
                        .cloned()
                        .ok_or(CommandError::TrackGone)?;
                    child.start += compound.start - compound.source_start;
                    parent.clips.push(Arc::new(child));
                }
                parent.clips.retain(|clip| clip.id != clip_id);
                parent.id.clone()
            };
            project.timelines.remove(internal_index);
            project.active_timeline_id = parent_id;
            Ok(Outcome {
                created_id: None,
                applied: true,
            })
        }

        _ => unreachable!("commands::apply routes only compound commands here"),
    }
}
