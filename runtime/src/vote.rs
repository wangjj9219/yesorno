use parity_codec::{Decode, Encode};
use support::{StorageValue, StorageMap, dispatch::Result, decl_module, decl_storage, decl_event, ensure};
use system::ensure_signed;
//use runtime_primitives::traits::{As, Zero, Hash, Saturating, CheckedSub, CheckedAdd, CheckedMul, CheckedDiv};
use rstd::vec::Vec;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Project {
    name: Vec<u8>,
    voted_members: u64,
    accumulative_weight: u64,
}

pub trait Trait: timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event! (
    pub enum Event<T>
    where AccountId = <T as system::Trait>::AccountId,
    {
        AddMember(AccountId, bool),
        AddProject(Vec<u8>),
        Vote(AccountId, u64, bool),
        CanVote(bool),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Vote {
        MembersCount get(members_count): u64;
        MembersArray get(members_array): map u64 => T::AccountId;
        MemberIsReviewer get(member_is_reviewer): map T::AccountId => Option<bool>;

        ProjectsCount get(projects_count): u64;
        ProjectsArray get(projects_array): map u64 => Project;

        Votes get(votes): map (u64, T::AccountId) => Option<bool>;

        // config
        Owner get(owner) config(): T::AccountId;
        CanVote get(can_vote) config(): bool;
        ReviewerWeight get(reviewer_weight) config(): u64;
        PlayerWeight get(player_weight) config(): u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        pub fn register_member(origin, account: T::AccountId, is_reviewer: bool) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(Self::owner() == sender, "Sender is not the owner!");
            ensure!(!<MemberIsReviewer<T>>::exists(account.clone()), "The member already exists!");
            
            <MembersArray<T>>::insert(Self::members_count(), account.clone());
            <MemberIsReviewer<T>>::insert(account.clone(), is_reviewer);
            <MembersCount<T>>::mutate(|n| *n += 1);

            Self::deposit_event(RawEvent::AddMember(account, is_reviewer));
            Ok(())
        }

        pub fn add_project(origin, name: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(Self::owner() == sender, "Sender is not the owner!");

            let project = Project {
                name: name.clone(),
                voted_members: 0,
                accumulative_weight: 0,
            };

            <ProjectsArray<T>>::insert(Self::projects_count(), project);
            <ProjectsCount<T>>::mutate(|n| *n += 1);

            Self::deposit_event(RawEvent::AddProject(name));
            Ok(())
        }

        pub fn vote(origin, project_id: u64, like: bool) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<MemberIsReviewer<T>>::exists(sender.clone()), "Sender must be reviewer or captain!");
            ensure!(<ProjectsArray<T>>::exists(project_id), "Project id must exist!");
            ensure!(Self::can_vote(), "Not in voting period!");
            ensure!(!<Votes<T>>::exists((project_id, sender.clone())), "Sender has already vote for this project!");

            let mut project = Self::projects_array(project_id);
            let added_weight = match (like, Self::member_is_reviewer(sender.clone()).ok_or("invalid vote")?) {
                (true, true) => Self::reviewer_weight(),
                (true, false) => Self::player_weight(),
                _ => 0,
            };
            project.accumulative_weight = project.accumulative_weight + added_weight;
            project.voted_members += 1;

            <Votes<T>>::insert((project_id, sender.clone()), like);
            <ProjectsArray<T>>::insert(project_id, project);
            
            Self::deposit_event(RawEvent::Vote(sender, project_id, like));
            Ok(())
        }

        pub fn switch_votable(origin) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(Self::owner() == sender, "Sender is not the owner!");

            let can_vote = !Self::can_vote();
            <CanVote<T>>::put(can_vote);

            Self::deposit_event(RawEvent::CanVote(can_vote));
            Ok(())
        }
    }
}