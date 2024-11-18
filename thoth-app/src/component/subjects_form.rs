use std::str::FromStr;
use thoth_api::model::subject::Subject;
use thoth_api::model::subject::SubjectType;
use thoth_errors::ThothError;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormSubjectTypeSelect;
use crate::component::utils::FormTextInput;
use crate::models::subject::create_subject_mutation::CreateSubjectRequest;
use crate::models::subject::create_subject_mutation::CreateSubjectRequestBody;
use crate::models::subject::create_subject_mutation::PushActionCreateSubject;
use crate::models::subject::create_subject_mutation::PushCreateSubject;
use crate::models::subject::create_subject_mutation::Variables;
use crate::models::subject::delete_subject_mutation::DeleteSubjectRequest;
use crate::models::subject::delete_subject_mutation::DeleteSubjectRequestBody;
use crate::models::subject::delete_subject_mutation::PushActionDeleteSubject;
use crate::models::subject::delete_subject_mutation::PushDeleteSubject;
use crate::models::subject::delete_subject_mutation::Variables as DeleteVariables;
use crate::models::subject::subject_types_query::FetchActionSubjectTypes;
use crate::models::subject::subject_types_query::FetchSubjectTypes;
use crate::models::subject::SubjectTypeValues;
use crate::string::CANCEL_BUTTON;
use crate::string::EMPTY_SUBJECTS;
use crate::string::REMOVE_BUTTON;

use super::ToElementValue;

pub struct SubjectsFormComponent {
    data: SubjectsFormData,
    new_subject: Subject,
    show_add_form: bool,
    fetch_subject_types: FetchSubjectTypes,
    push_subject: PushCreateSubject,
    delete_subject: PushDeleteSubject,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct SubjectsFormData {
    subject_types: Vec<SubjectTypeValues>,
}

pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetSubjectTypesFetchState(FetchActionSubjectTypes),
    GetSubjectTypes,
    SetSubjectPushState(PushActionCreateSubject),
    CreateSubject,
    SetSubjectDeleteState(PushActionDeleteSubject),
    DeleteSubject(Uuid),
    ChangeSubjectType(SubjectType),
    ChangeCode(String),
    ChangeOrdinal(String),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub subjects: Option<Vec<Subject>>,
    pub work_id: Uuid,
    pub update_subjects: Callback<Option<Vec<Subject>>>,
}

impl Component for SubjectsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let data: SubjectsFormData = Default::default();
        let show_add_form = false;
        let new_subject: Subject = Default::default();
        let push_subject = Default::default();
        let delete_subject = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        ctx.link().send_message(Msg::GetSubjectTypes);

        SubjectsFormComponent {
            data,
            new_subject,
            show_add_form,
            fetch_subject_types: Default::default(),
            push_subject,
            delete_subject,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleAddFormDisplay(value) => {
                self.show_add_form = value;
                true
            }
            Msg::SetSubjectTypesFetchState(fetch_state) => {
                self.fetch_subject_types.apply(fetch_state);
                self.data.subject_types = match self.fetch_subject_types.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.subject_types.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetSubjectTypes => {
                ctx.link().send_future(
                    self.fetch_subject_types
                        .fetch(Msg::SetSubjectTypesFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetSubjectTypesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetSubjectPushState(fetch_state) => {
                self.push_subject.apply(fetch_state);
                match self.push_subject.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_subject {
                        Some(p) => {
                            let subject = p.clone();
                            let mut subjects: Vec<Subject> =
                                ctx.props().subjects.clone().unwrap_or_default();
                            subjects.push(subject);
                            ctx.props().update_subjects.emit(Some(subjects));
                            ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                            true
                        }
                        None => {
                            ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateSubject => {
                let body = CreateSubjectRequestBody {
                    variables: Variables {
                        work_id: ctx.props().work_id,
                        subject_type: self.new_subject.subject_type,
                        subject_code: self.new_subject.subject_code.clone(),
                        subject_ordinal: self.new_subject.subject_ordinal,
                    },
                    ..Default::default()
                };
                let request = CreateSubjectRequest { body };
                self.push_subject = Fetch::new(request);
                ctx.link()
                    .send_future(self.push_subject.fetch(Msg::SetSubjectPushState));
                ctx.link()
                    .send_message(Msg::SetSubjectPushState(FetchAction::Fetching));
                false
            }
            Msg::SetSubjectDeleteState(fetch_state) => {
                self.delete_subject.apply(fetch_state);
                match self.delete_subject.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_subject {
                        Some(subject) => {
                            let to_keep: Vec<Subject> = ctx
                                .props()
                                .subjects
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|s| s.subject_id != subject.subject_id)
                                .collect();
                            ctx.props().update_subjects.emit(Some(to_keep));
                            true
                        }
                        None => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::DeleteSubject(subject_id) => {
                let body = DeleteSubjectRequestBody {
                    variables: DeleteVariables { subject_id },
                    ..Default::default()
                };
                let request = DeleteSubjectRequest { body };
                self.delete_subject = Fetch::new(request);
                ctx.link()
                    .send_future(self.delete_subject.fetch(Msg::SetSubjectDeleteState));
                ctx.link()
                    .send_message(Msg::SetSubjectDeleteState(FetchAction::Fetching));
                false
            }
            Msg::ChangeSubjectType(val) => self.new_subject.subject_type.neq_assign(val),
            Msg::ChangeCode(code) => self
                .new_subject
                .subject_code
                .neq_assign(code.trim().to_owned()),
            Msg::ChangeOrdinal(ordinal) => {
                let ordinal = ordinal.parse::<i32>().unwrap_or(0);
                self.new_subject.subject_ordinal.neq_assign(ordinal);
                false // otherwise we re-render the component and reset the value
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut subjects = ctx.props().subjects.clone().unwrap_or_default();
        let open_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(true)
        });
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        subjects.sort_by(|a, b| {
            if a.subject_type == b.subject_type {
                a.subject_ordinal.partial_cmp(&b.subject_ordinal).unwrap()
            } else {
                a.subject_type.partial_cmp(&b.subject_type).unwrap()
            }
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Subjects" }
                </p>
                <div class="panel-block">
                    <button
                        class="button is-link is-outlined is-success is-fullwidth"
                        onclick={ open_modal }
                    >
                        { "Add Subject" }
                    </button>
                </div>
                <div class={ self.add_form_status() }>
                    <div class="modal-background" onclick={ &close_modal }></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "New Subject" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick={ &close_modal }
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id="subjects-form" onsubmit={ ctx.link().callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::CreateSubject
                            }) }
                            >
                                <FormSubjectTypeSelect
                                    label = "Subject Type"
                                    value={ self.new_subject.subject_type }
                                    data={ self.data.subject_types.clone() }
                                    onchange={ ctx.link().callback(|e: Event|
                                        Msg::ChangeSubjectType(SubjectType::from_str(&e.to_value()).unwrap())
                                    ) }
                                    required = true
                                />
                                <FormTextInput
                                    label = "Subject Code"
                                    value={ self.new_subject.subject_code.clone() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeCode(e.to_value())) }
                                    required = true
                                />
                                <FormNumberInput
                                    label = "Subject Ordinal"
                                    value={ self.new_subject.subject_ordinal }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeOrdinal(e.to_value())) }
                                    required = true
                                    min={ "1".to_string() }
                                />
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                type="submit"
                                form="subjects-form"
                            >
                                { "Add Subject" }
                            </button>
                            <button
                                class="button"
                                onclick={ &close_modal }
                            >
                                { CANCEL_BUTTON }
                            </button>
                        </footer>
                    </div>
                </div>
                {
                    if !subjects.is_empty() {
                        html!{{for subjects.iter().map(|p| self.render_subject(ctx, p))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_SUBJECTS }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl SubjectsFormComponent {
    fn add_form_status(&self) -> String {
        match self.show_add_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn render_subject(&self, ctx: &Context<Self>, s: &Subject) -> Html {
        let subject_id = s.subject_id;
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-tag" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Subject Type" }</label>
                        <div class="control is-expanded">
                            {&s.subject_type}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Subject Code" }</label>
                        <div class="control is-expanded">
                            {&s.subject_code.clone()}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Subject Ordinal" }</label>
                        <div class="control is-expanded">
                            {&s.subject_ordinal.clone()}
                        </div>
                    </div>

                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick={ ctx.link().callback(move |_| Msg::DeleteSubject(subject_id)) }
                            >
                                { REMOVE_BUTTON }
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
